use std::{io::{prelude::*, BufReader, Write}, net::TcpListener};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

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


type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// Size is the number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if size is 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(sender) = self.sender.as_ref() {
            if let Err(err) = sender.send(job) {
                println!("Error: {err}");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {0}", worker.id);

            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(_) => println!("Worker {0} stopped", worker.id),
                    Err(_) => println!("Something went wrong stopping worker {0}", worker.id),
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            match receiver
                .lock()
                .unwrap_or_else(|err| panic!("Error getting info from receiver: {err}"))
                .recv()
            {
                Ok(job) => {
                    println!("Worker {id} get a job. Starting...");
                    job();
                }
                Err(_) => {
                    println!("Session finished, terminating worker...");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener: TcpListener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let pool = ThreadPool::new(8);

    for stream in listener.incoming() {

        pool.execute(|| {
                let mut stream = stream.unwrap();

                let response = "HTTP/1.1 200 OK\r\n\r\n";
                let err_response = "HTTP/1.1 404 Not Found\r\n\r\n";

                let buf_reader = BufReader::new(&mut stream);


                let http_request: Vec<_> = buf_reader
                    .lines()
                    .map(|result| result.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();

                // let request_line = buf_reader.lines().next().unwrap().unwrap();

                let parts = http_request[0].split(" ").collect::<Vec<_>>();

                let user_agent = http_request.iter().find(|x| x.contains("User-Agent"));

                let http_req_line: HttpReqLine = HttpReqLine::new(parts[0], parts[1], parts[2], user_agent);

                if http_req_line.method == "GET" && http_req_line.target == "/" || http_req_line.target.contains("echo") || http_req_line.target.contains("user-agent") {
                    thread::sleep(Duration::from_secs(5));

                    if http_req_line.user_agent.is_some() && http_req_line.target.contains("user-agent") {
                        println!("user agent test here");
                        let user_agent_value = http_req_line.user_agent.unwrap().split(": ").collect::<Vec<_>>()[1];

                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {len}\r\n\r\n{body}", len = user_agent_value.len(), body = user_agent_value);

                        stream.write_all(response.as_bytes()).unwrap();
                    }       
                    
                    if http_req_line.target.contains("echo") {
                        let path = http_req_line.target.split("/").collect::<Vec<_>>()[2];

                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {len}\r\n\r\n{body}", len = path.len(), body = path);

                        stream.write_all(response.as_bytes()).unwrap();
                    }   else  {
                        println!("here bb");

                        stream.write_all(response.as_bytes()).unwrap();
                    }
                } else {
                    stream.write_all(err_response.as_bytes()).unwrap();
                }
        });

    }
}
