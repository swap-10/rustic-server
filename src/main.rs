use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use rustic_server::ThreadPool;

struct Config {
    address: String,
    port: String
}

struct Request {
    method: String,
    path: String,
    protocol: String
}

struct Response {
    protocol: String,
    status: String,
    body: String
}

struct RequestHandler {
    request: Request,
    response: Response
}

// Implements the config
impl Config {
    // Create a new instance of Config
    fn new() -> Config {
        Config {
            address: String::from(""),
            port: String::from(""),
        }
    }

    // Parse the command line arguments
    fn parse_args(&mut self, args: &Vec<String>) {

        self.address = args[1].clone();

        self.port = args[2].clone();
    }
}

// Implements the RequestHandler
impl RequestHandler {
    // Creates a new instance of RequestHandler
    fn new() -> RequestHandler {
        RequestHandler {
            request: Request {
                method: String::from(""),
                path: String::from(""),
                protocol: String::from("")
            },
            response: Response {
                protocol: String::from("HTTP/1.1"),
                status: String::from("200 OK"),
                body: String::from("")
            }
        }
    }

    // parses the request string and populates the Request struct
    fn parse_request(&mut self, request: &String) {
        let mut lines = request.lines();
        let mut line = lines.next().unwrap().split_whitespace();


        // Get the request method
        self.request.method = String::from(line.next().unwrap());

        // Get the request path and add the static folder path to it
        let mut path = String::from("./static");
        path.push_str(line.next().unwrap());
        
        if path.ends_with("/") {
            path.pop();
        }

        self.request.path = path;

        // Get the request protocol
        self.request.protocol = String::from(line.next().unwrap());

    }

    
    // get the response, set the status and body    
    fn get_response(&mut self) {

        if self.request.method == "GET" {
            // Check if the path exists
            if fs::metadata(&self.request.path).is_ok() {
                // Get the file contents
                self.response.body = fs::read_to_string(&self.request.path).unwrap();
            } else {
                // Set the status to 404
                self.response.status = String::from("404 NOT FOUND");

                self.response.body = fs::read_to_string("./static/404.html").unwrap();
            }
        } else {

            self.response.status = String::from("405 METHOD NOT ALLOWED");

            self.response.body = fs::read_to_string("./static/404.html").unwrap();
        }
        
        self.response.body = format!("{} {} {}\r\n\r\n{}", self.response.protocol, self.response.status, self.response.body.len(), self.response.body);

    }
}



fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let mut request_handler = RequestHandler::new();
    request_handler.parse_request(&String::from_utf8_lossy(&buffer[..]).to_string());

    request_handler.get_response();

    stream.write(request_handler.response.body.as_bytes()).unwrap();

    // Flush the stream
    stream.flush().unwrap();
}


fn main() {

    let mut config = Config::new();

    config.parse_args(&std::env::args().collect());

    let thread_pool = ThreadPool::new(5);

    // Create a new instance of TcpListener
    let listener = TcpListener::bind(format!("{}:{}", config.address, config.port)).unwrap();

    for stream in listener.incoming() {
        // Get the stream
        let stream_obj = stream.unwrap();

        thread_pool.execute(|| {
            handle_connection(stream_obj);
        });
    }
}