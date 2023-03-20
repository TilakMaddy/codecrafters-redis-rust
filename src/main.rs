use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("accepted new connection");
                thread::spawn(|| handle_connection(tcp_stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let client_addr = stream.local_addr().unwrap().ip().to_string();

    // As of now, we don't care what we receive so we hard code input
    let binding = craft_response(
        "*1\r\n$4\r\nping\r\n".to_string()
    );

    let mut reading_iterator =
        buf_reader.lines()
            .map(|line| line.unwrap());

    while let Some(x) = reading_iterator.next() {
        if x.is_empty() {
            break;
        }
        tcp_stream.write("+PONG\r\n".as_bytes()).unwrap();
    }

}

fn craft_response(request_string: String) -> String {

    let mut decomposed: Vec<&str> = request_string.split("\r\n").collect();
    decomposed.pop(); // Getting rid of the blank "" due to the last CR-LF

    let op: Operation = decomposed.into();
    return match op {
        Operation::Pong(response_data) => {
            response_data.join("\r\n")
        }
    }

}

#[derive(PartialEq, Debug)]
enum Operation {
    Pong(Vec<String>)
}

impl From<Vec<&str>> for Operation {

    fn from(mut data: Vec<&str>) -> Self {
        if data == &["*1", "$4", "ping"] {
            return Operation::Pong(vec![
                "$4".to_string(),
                "PONG".to_string(),
                "".to_string()
            ]);
        }

        let mut d = data[0].to_string();
        let data_len: String = d.drain(1..d.len()).collect();
        let data_len = data_len.parse::<usize>().unwrap();

        let mut response_data = vec![];
        response_data.push(format!("*{}", data_len - 1));
        let rest: Vec<_> = data.drain(3..data.len()).collect();

        for elem in rest {
            response_data.push(elem.to_string());
        }

        Operation::Pong(response_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::Operation;

    #[test]
    fn check_decomposition_only_ping() {
        let req = "*1\r\n$4\r\nping\r\n";

        let mut decomposed: Vec<&str> = req.split("\r\n").collect();
        assert_eq!(decomposed, &["*1", "$4", "ping", ""]);

        decomposed.pop();
        assert_eq!(decomposed, &["*1", "$4", "ping"]);

        let op: Operation = decomposed.into();
        assert_eq!(op, Operation::Pong(vec![
                "*1".to_string(),
                "$4".to_string(),
                "pong".to_string()
            ])
        );
    }

    #[test]
    fn check_decomposition_ping_with_words() {

        let mut decomposed: Vec<&str> = vec![
            "*2", "$4", "ping", "$3", "cat"
        ];

        let op: Operation = decomposed.into();
        assert_eq!(op, Operation::Pong(
            vec![
                "*1".to_string(),
                "$3".to_string(),
                "cat".to_string()
            ]
        ));

    }

}

