use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use rustic_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9393").unwrap();
    let thread_pool = ThreadPool::new(5);

    for stream in listener.incoming() {
        let stream_object = stream.unwrap();
        
        thread_pool.execute(|| {
            handle_connection(stream_object);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "static/index.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "static/404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    
    stream.write_all(response.as_bytes()).unwrap();
}