use std::{io::{prelude::*, BufReader, Write}, net::TcpListener};

#[derive(Debug)]
struct HttpReqLine<'a> {
    method: &'a str,
    target: &'a str,
    http_version: &'a str,
    user_agent: Option<&'a String>
}

impl <'a> HttpReqLine<'a> {
    fn new(method: &'a str, target: &'a str, http_version: &'a str, user_agent: Option<&'a String>) -> Self {
        Self { method, target, http_version, user_agent }
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

                let buf_reader: BufReader<&mut std::net::TcpStream> = BufReader::new(&mut stream);

                let http_request: Vec<_> = buf_reader
                    .lines()
                    .map(|result| result.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();

                println!("here {:?}", http_request);
                // let request_line = buf_reader.lines().next().unwrap().unwrap();

                let parts = http_request[0].split(" ").collect::<Vec<_>>();

                let user_agent = http_request.iter().find(|x| x.contains("User-Agent"));

                let http_req_line: HttpReqLine = HttpReqLine::new(parts[0], parts[1], parts[2], user_agent);

                if http_req_line.method == "GET" && http_req_line.target == "/" || http_req_line.target.contains("echo") || http_req_line.target.contains("user-agent") {
                    if http_req_line.user_agent.is_some() && http_req_line.target.contains("user-agent") {
                        let user_agent_value = http_req_line.user_agent.unwrap().split(": ").collect::<Vec<_>>()[1];

                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {len}\r\n\r\n{body}", len = user_agent_value.len(), body = user_agent_value);

                        stream.write_all(response.as_bytes()).unwrap();
                    }       
                    
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
