use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("accepted new connection");
                handle_connection(tcp_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&stream);

    let mut data = String::new();
    let _request = buf_reader.read_to_string(&mut data)
        .expect("Couldn't read from buffer !");

    println!("[Recv] : {}", data);

    let binding = craft_response(data);
    let response = binding.as_bytes();
    stream.write_all(response)
        .expect("Couldn't reply to client !");

    println!("[Sent] : {}", String::from_utf8_lossy(response));
}

fn craft_response(request_string: String) -> String {

    let decomposed = request_string.split_once(" ");
    let default_response = "+PONG\r\n".to_string();

    if let Some(message) = decomposed {
        if message.1.len() == 0 {

            return default_response;
        }
        let size = message.1.as_bytes().len();
        let response = format!("${}\r\n{}\r\n", size, message.1);
        return response;
    }

    return default_response;
}

