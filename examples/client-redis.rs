use mini_redis::{client, Result};
use std::io;
#[tokio::main]
async fn main() -> Result<()> {
    // open a connection to the mini redis server.
    let mut client = client::connect("127.0.0.1:6969").await?;
    
    loop {
        // set{value:key}

        let mut input=String::new();
        io::stdin().read_line(&mut input).expect("error reading the message");
        let key=input.trim().split(':').nth(0).unwrap().to_string();
        let value=input.trim().split(':').nth(1).unwrap().to_string();
        client.set(&key, value.into()).await?;

        //retrieve a the same data
        //i am going to work on a cli tool with parsing later
        let result = client.get(&key).await?;
        if let Some(bytes) = result {
        let string_result = std::str::from_utf8(&bytes).unwrap();
        println!("got value from the server; result={}", string_result);
        }
    
    }
    //to make sure the return type is satisfied
    Ok(())
}