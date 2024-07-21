use std::{io::{prelude::*, BufReader, Write}, net::TcpListener};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener: TcpListener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                let err_response = "HTTP/1.1 404 Not Found\r\n\r\n";

                let buf_reader = BufReader::new(&mut stream);
                let request_line = buf_reader.lines().next().unwrap().unwrap();

                if request_line == "GET / HTTP/1.1" {
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    stream.write_all(err_response.as_bytes()).unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
