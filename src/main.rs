use crate::config::redis_defaults;
use crate::connection::Connection;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};

mod config;
mod connection;
mod frame;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;
type ExpDb = Arc<Mutex<HashMap<String, Instant>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(redis_defaults())
        .await
        .expect("Failed to bind tcp listener ");

    println!("Listening for incoming connections on {}", redis_defaults());

    let db = Arc::new(Mutex::new(HashMap::new()));
    let exp = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .expect("Failed to accept client's request ");

        let db = db.clone();
        let exp = exp.clone();

        tokio::spawn(async move {
            process_client(stream, db, exp).await;
        });
    }
}

async fn process_client(stream: TcpStream, db: Db, exp: ExpDb) {
    let mut conn = Connection::new(stream);
    conn.run_read_write_loop(db.clone(), exp.clone()).await;
}
