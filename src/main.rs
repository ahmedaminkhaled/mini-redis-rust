use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use mini_redis::{frame, Connection, Frame};
type Shardeddb=Arc<Vec<Mutex<HashMap<String,Vec<u8>>>>>;
use bytes::Bytes;
//number of shards
const N:u8=32;

//our local host
const ADDRESS:&str="127.0.0.1:6969";
//creates a sharded db using arc and mutex
fn new_sharded_db()->Shardeddb{
    let mut db=Vec::with_capacity(N as usize);
    for _ in 0..N{
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}
//indexing the db vector using a hashing algorithm
fn index(key:&str)->usize{
    let mut hasher=DefaultHasher::new();
    key.hash(&mut hasher);
    (hasher.finish() as usize)%(N as usize)
}


#[tokio::main]
async fn main(){
    //binding a tcplistener and create a new db
    let listener=TcpListener::bind(ADDRESS).await.unwrap();
    let db=new_sharded_db();
    loop{
        //average tokio task and socket handling
        let (socket,_)=listener.accept().await.unwrap();
        let db=db.clone();
        tokio::spawn(async move{
            process(socket,db).await
        });
    }
}
async fn process(socket: TcpStream,db:Shardeddb) {
    use mini_redis::Command::{self, Get, Set};
    //create a connection wrapper to the tcp socket
    let mut connection=Connection::new(socket);
    //Keep reading frames
    while let Some(frame)=connection.read_frame().await.unwrap(){
        //parsing the frame into a redis command
        let response=match Command::from_frame(frame).unwrap() {
            Set(cmd)=>{
                //getting a shard based on the index and adding the key,value
                let mut shard=db[index(cmd.key())].lock().unwrap();
                shard.insert(cmd.key().to_string(), cmd.value().clone().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd)=>{
                //same logic just getting the value from the key 
                let idx=index(cmd.key());
                let shard=db[idx].lock().unwrap();
                //if the key is valid we return a value if it isnt we return a Null frame
                if let Some(value)=shard.get(cmd.key()){
                    Frame::Bulk(Bytes::from(value.clone()))
                }
                else{
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        //writing the frame
        connection.write_frame(&response).await.unwrap();
    }
    
}





/*
my attempt at a sharedHashmap construct 
#[derive(Clone)]
pub struct SharedHashmap{
    inner:Arc<Mutex<Sharedmapinner>>,
}
pub struct Sharedmapinner{
    data:HashMap<String,Bytes>,
}
impl SharedHashmap {
    fn new()->Self{
        let hm=Sharedmapinner{
            data:HashMap::new(),
        };
        Self { inner: Arc::new(Mutex::new(hm)) }
    }
    fn insert(&self,key:String,value:Bytes){
        let mut lock=self.inner.lock().unwrap();
        lock.data.insert(key, value);
    }
    fn get(&self,key:String)->Option<Bytes>{
        let lock=self.inner.lock().unwrap();
        lock.data.get(&key).cloned()
    }
} */