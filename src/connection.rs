use bytes::{Bytes, BytesMut,Buf};
use mini_redis::frame;
use tokio::net::{TcpListener,TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt,BufWriter};
use mini_redis::{Frame, Result,};
use std::io::{self, Cursor};



pub struct Connection{
    stream:BufWriter<TcpStream>,
    buffer:BytesMut,
}
impl Connection{
    pub  fn new(stream:TcpStream)->Self{
        let  buffer=BytesMut::with_capacity(4096);
        Self{stream:BufWriter::new(stream),buffer:buffer}
    }
    pub async fn read_frame(&mut self)->Result<Option<Frame>>{
    loop {
            
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }
    pub fn parse_frame(&mut self)->Result<Option<Frame>>{
        let mut buf=Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(_)=>{
                let len=buf.position() as usize;
                buf.set_position(0);
                let frame=Frame::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(frame))


            }
            Err(frame::Error::Incomplete)=>Ok(None),
            Err(e)=>Err(e.into()),
        }
    }
    pub async fn write_frame(&mut self,frame:&Frame)->io::Result<()>{
        match frame {
            Frame::Simple(val)=>{
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                
            }
            Frame::Error(val)=>{
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                
            }
            Frame::Integer(val)=>{
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
                
                
            }
            Frame::Null => {
            self.stream.write_all(b"$-1\r\n").await?;
            
            }
            Frame::Bulk(val) => {
            let len = val.len();

            self.stream.write_u8(b'$').await?;
            self.write_decimal(len as u64).await?;
            self.stream.write_all(val).await?;
            self.stream.write_all(b"\r\n").await?;
        }
            Frame::Array(items) => {
                self.stream.write_u8(b'*').await?;
                self.write_decimal(items.len() as u64).await?;
                
                
                for item in items {
                    Box::pin(self.write_frame(item)).await?;
                }
            }
            
        }
        self.stream.flush().await?;
        Ok(())
    }
    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    use tokio::net::{TcpListener, TcpStream};
    use mini_redis::Frame;
    use bytes::Bytes;

    async fn create_connection_pair() -> (Connection, Connection) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client_stream = TcpStream::connect(addr).await.unwrap();
        let (server_stream, _) = listener.accept().await.unwrap();

        let client_conn = Connection::new(client_stream);
        let server_conn = Connection::new(server_stream);

        (client_conn, server_conn)
    }

    #[tokio::test]
    async fn test_write_read_simple_frame() {
        let (mut client, mut server) = create_connection_pair().await;

        // Write a simple frame
        let frame = Frame::Simple("OK".to_string());
        client.write_frame(&frame).await.unwrap();

        // Read it back
        let received = server.read_frame().await.unwrap();
        match received {
            Some(Frame::Simple(s)) => assert_eq!(s, "OK"),
            _ => panic!("Expected Simple frame"),
        }
    }

    #[tokio::test]
    async fn test_write_read_bulk_frame() {
        let (mut client, mut server) = create_connection_pair().await;

        // Write a bulk frame
        let data = Bytes::from("hello world");
        let frame = Frame::Bulk(data.clone());
        client.write_frame(&frame).await.unwrap();

        // Read it back
        let received = server.read_frame().await.unwrap();
        match received {
            Some(Frame::Bulk(bytes)) => assert_eq!(bytes, data),
            _ => panic!("Expected Bulk frame"),
        }
    }

    #[tokio::test]
    async fn test_write_read_array_frame() {
        let (mut client, mut server) = create_connection_pair().await;

        // Write an array frame
        let frame = Frame::Array(vec![
            Frame::Simple("SET".to_string()),
            Frame::Bulk(Bytes::from("key")),
            Frame::Bulk(Bytes::from("value")),
        ]);
        client.write_frame(&frame).await.unwrap();

        // Read it back
        let received = server.read_frame().await.unwrap();
        match received {
            Some(Frame::Array(items)) => {
                assert_eq!(items.len(), 3);
                // Check first item
                match &items[0] {
                    Frame::Simple(s) => assert_eq!(s, "SET"),
                    _ => panic!("Expected Simple frame"),
                }
            },
            _ => panic!("Expected Array frame"),
        }
    }
}