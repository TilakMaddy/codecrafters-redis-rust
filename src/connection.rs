use std::io::Cursor;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use crate::Db;
use crate::frame::Frame;

pub struct Connection {
    pub stream: BufWriter<TcpStream>,
    pub buffer: BytesMut
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        let redis_conn = Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024)
        };
        return redis_conn;
    }

    pub async fn run_read_write_loop(&mut self, db: Db) {
        loop {
            let read_bytes = self.stream.read_buf(&mut self.buffer).await
                .expect("failed to read bytes into buffer ");

            if read_bytes == 0 {
                break;
            }

            let mut buf = Cursor::new(&self.buffer[..]);
            let frame =  Frame::parse(&mut buf);
            println!("Received {:?}", frame);
            send_response(frame, self, db.clone()).await;

            self.buffer.advance(read_bytes);
        }
    }

}

async fn send_response(frame: Frame, conn: &mut Connection, db: Db) {

    let mut bytes_to_write:Vec<Vec<u8>> = vec![];

    if let Frame::Array(items) = frame {
        let args: Vec<_> = items.clone().into_iter().skip(1).collect();
        if let Frame::Bulk(cmd_bytes) = items.get(0).unwrap() {
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
                    if let Frame::Bulk(key_bytes) = key {
                        if let Frame::Bulk(value_bytes) = value {
                            let key_str = String::from_utf8(key_bytes.to_vec()).unwrap();
                            {
                                let mut db = db.lock().unwrap();
                                db.insert(key_str, value_bytes);
                            }
                            bytes_to_write.push(b"+OK\r\n".to_vec());
                        }
                    }
                    else { panic!("invalid protocol format !") }
                }
                "get" => {
                    let key = args.get(0).unwrap().clone();
                    if let Frame::Bulk(key_bytes) = key {
                        let key_str = String::from_utf8(key_bytes.to_vec()).unwrap();
                        let db = db.lock().unwrap();

                        if let Some(val) = db.get(key_str.as_str()).clone() {
                            let len = val.len();
                            let ret_val = format!("${}\r\n", len);
                            bytes_to_write.push(ret_val.as_bytes().to_vec());
                            bytes_to_write.push(val.to_vec());
                            bytes_to_write.push(b"\r\n".to_vec());
                        } else {
                            bytes_to_write.push(b"-1\r\n".to_vec());
                        }
                    }
                    else { panic!("invalid protocol format !") }
                }
                _ => panic!("command not implemented ")
            }
        }
        else { panic!("invalid protocol format !"); }
    }
    else { panic!("Not ready !") }

    // write response to the connection stream
    for group in bytes_to_write {
        conn.stream.write_all(group.as_ref()).await.unwrap();
    }

}
