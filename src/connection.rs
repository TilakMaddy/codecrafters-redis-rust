use crate::frame::Frame;
use crate::{Db, ExpDb};
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

pub struct Connection {
    pub stream: BufWriter<TcpStream>,
    pub buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        let redis_conn = Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        };
        return redis_conn;
    }

    pub async fn run_read_write_loop(&mut self, db: Db, exp: ExpDb) {
        loop {
            let read_bytes = self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .expect("failed to read bytes into buffer ");

            if read_bytes == 0 {
                break;
            }

            let mut buf = Cursor::new(&self.buffer[..]);
            let frame = Frame::parse(&mut buf);
            println!("Received {:?}", frame);
            send_response(frame, self, db.clone(), exp.clone()).await;

            self.buffer.advance(read_bytes);
        }
    }
}

async fn send_response(frame: Frame, conn: &mut Connection, db: Db, exp: ExpDb) {
    let mut bytes_to_write: Vec<Vec<u8>> = vec![];

    let items = frame.unwrap_array();
    let args: Vec<_> = items.clone().into_iter().skip(1).collect();
    let cmd_bytes = items.get(0).unwrap().unwrap_bulk();

    let cmd = String::from_utf8(cmd_bytes.to_vec()).unwrap();
    match cmd.to_ascii_lowercase().as_ref() {
        "ping" => {
            bytes_to_write.push(b"+PONG\r\n".to_vec());
        }
        "echo" => {
            let write_data = args.first().unwrap().clone().encode();
            bytes_to_write.push(write_data.as_bytes().to_vec());
        }
        "set" => {
            let key = args.get(0).unwrap().clone();
            let value = args.get(1).unwrap().clone();

            let key_str = key.unwrap_bulk_as_string();
            let value_bytes = value.unwrap_bulk();

            let px = args.get(2);
            let milli_seconds = args.get(3);

            if let Some(Frame::Bulk(_px)) = px {
                let milli_seconds = milli_seconds.unwrap().unwrap_bulk_as_string();
                let milli_seconds: u64 = milli_seconds.parse().unwrap();

                let mut exp = exp.lock().unwrap();
                exp.insert(
                    key_str.clone(),
                    Instant::now() + Duration::from_millis(milli_seconds)
                );
            }
            let mut db = db.lock().unwrap();
            db.insert(key_str.clone(), value_bytes);
            bytes_to_write.push(b"+OK\r\n".to_vec());
        }
        "get" => {
            let key = args.get(0).unwrap().clone();
            let key_str = key.unwrap_bulk_as_string();

            let mut db = db.lock().unwrap();
            let mut exp = exp.lock().unwrap();

            if let Some(expire_time) = exp.get(&key_str) {
                if Instant::now() >= *expire_time {
                    println!("Key has been expired !");
                    db.remove(key_str.as_str());
                    exp.remove(key_str.as_str());
                } else {
                    println!("Key has not been expired !");
                }
            }

            if let Some(val) = db.get(key_str.as_str()) {
                let len = val.len();
                let ret_val = format!("${}\r\n", len);
                bytes_to_write.push(ret_val.as_bytes().to_vec());
                bytes_to_write.push(val.to_vec());
                bytes_to_write.push(b"\r\n".to_vec());
            } else {
                bytes_to_write.push(b"-1\r\n".to_vec());
            }
        }
        _ => panic!("command not implemented "),
    }

    // write response to the connection stream
    for group in bytes_to_write {
        conn.stream.write_all(group.as_ref()).await.unwrap();
    }
    conn.stream.flush().await.unwrap();
}
