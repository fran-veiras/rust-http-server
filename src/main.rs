use std::{io::{prelude::*, BufReader, Write}, net::TcpListener};

#[derive(Debug)]
struct HttpReqLine<'a> {
    method: &'a str,
    target: &'a str,
    http_version: &'a str
}

impl <'a> HttpReqLine<'a> {
    fn new(method: &'a str, target: &'a str, http_version: &'a str) -> Self {
        Self { method, target, http_version }
    }
}

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

                let parts = request_line.split(" ").collect::<Vec<_>>();


                let http_req_line = HttpReqLine::new(parts[0], parts[1], parts[2]);

                if http_req_line.method == "GET" && http_req_line.target == "/" || http_req_line.target.contains("echo") {
                    if http_req_line.target.contains("echo") {
                        let path = http_req_line.target.split("/").collect::<Vec<_>>()[2];

                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {len}\r\n\r\n{body}", len = path.len(), body = path);

                        stream.write_all(response.as_bytes()).unwrap();
                    }   else  {
                        stream.write_all(response.as_bytes()).unwrap();
                    }
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
