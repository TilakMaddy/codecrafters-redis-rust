use tokio::net::{TcpListener, TcpStream};
use crate::config::redis_defaults;
use crate::connection::Connection;

mod config;
mod connection;
mod frame;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(redis_defaults()).await
        .expect("Failed to bind tcp listener ");

    println!("Listening for incoming connections on {}", redis_defaults());

    loop {
        let (stream, _) = listener.accept().await
            .expect("Failed to accept client's request ");

        tokio::spawn(async move {
            process_client(stream).await;
        });
    }
}

async fn process_client(stream: TcpStream) {
    let mut conn = Connection::new(stream);
    conn.run_read_write_loop().await;
}
