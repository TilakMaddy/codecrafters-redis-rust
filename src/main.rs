use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use tokio::net::{TcpListener, TcpStream};
use crate::config::redis_defaults;
use crate::connection::Connection;

mod config;
mod connection;
mod frame;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(redis_defaults()).await
        .expect("Failed to bind tcp listener ");

    println!("Listening for incoming connections on {}", redis_defaults());

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await
            .expect("Failed to accept client's request ");

        let db = db.clone();

        tokio::spawn(async move {
            process_client(stream, db).await;
        });
    }
}

async fn process_client(stream: TcpStream, db: Db) {
    let mut conn = Connection::new(stream);
    conn.run_read_write_loop(db.clone()).await;
}
