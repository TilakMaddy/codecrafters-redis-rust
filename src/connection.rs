use std::io::Cursor;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
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

    pub async fn run_read_write_loop(&mut self) {
        loop {
            let read_bytes = self.stream.read_buf(&mut self.buffer).await
                .expect("failed to read bytes into buffer ");

            if read_bytes == 0 {
                break;
            }

            let mut buf = Cursor::new(&self.buffer[..]);
            let frame =  Frame::parse(&mut buf);
            println!("Received {:?}", frame);
            send_response(frame, self).await;

            self.buffer.advance(read_bytes);
        }
    }

}

async fn send_response(frame: Frame, conn: &mut Connection) {
    if let Frame::Array(items) = frame {
        let args: Vec<_> = items.clone().into_iter().skip(1).collect();
        if let Frame::Bulk(cmd_bytes) = items.get(0).unwrap() {
            let cmd = String::from_utf8(cmd_bytes.to_vec()).unwrap();
            match cmd.to_ascii_lowercase().as_ref() {
                "ping" => {
                    let _ = conn.stream.write(b"+PONG\r\n").await.unwrap();
                    let _ = conn.stream.flush().await;
                }
                "echo" => {
                    let write_data = args.first().unwrap().clone().encode();
                    conn.stream.write(write_data.as_bytes()).await.unwrap();
                    let _ = conn.stream.flush().await;
                }
                _ => panic!("command not implemented ")
            }
        }
        else { panic!("invalid protocol format !"); }
    }
    else { panic!("Not ready !") }
}
