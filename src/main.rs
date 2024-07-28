use std::{io::{prelude::*, BufReader, Write}, net::TcpListener, thread, time::Duration};

use http_server_starter_rust::ThreadPool;

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
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream: std::net::TcpStream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream : std::net::TcpStream) {
    let err_response: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

    let buf_reader = BufReader::new(&mut stream);

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let parts = http_request[0].split(" ").collect::<Vec<_>>();

    let user_agent = http_request.iter().find(|x| x.contains("User-Agent"));

    let http_req_line: HttpReqLine = HttpReqLine::new(parts[0], parts[1], parts[2], user_agent);

    if http_req_line.method == "GET" && http_req_line.target == "/" || http_req_line.target.contains("echo") || http_req_line.target.contains("user-agent") {
        let (status_line, response_lenght, contents) = match { 
            if http_req_line.target.contains("user-agent") {
                "user-agent"
            } else if http_req_line.target.contains("echo") {
                "echo"
            } else {
                "not-found"
            }} {
            "user-agent" => {
                let user_agent_value: &str = http_req_line.user_agent.unwrap().split(": ").collect::<Vec<_>>()[1];

                ("HTTP/1.1 200 OK", user_agent_value.len(), user_agent_value)
            }
            "echo" => {
                thread::sleep(Duration::from_secs(10));
                let path = http_req_line.target.split("/").collect::<Vec<_>>()[2];

                ("HTTP/1.1 200 OK", path.len(), path)
            }
            _ => ("HTTP/1.1 200 OK", 0, ""),
        };

        let response: String =
            format!("{status_line}\r\nContent-Length: {response_lenght}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).unwrap();
    } else {
        stream.write_all(err_response.as_bytes()).unwrap();
    }
}