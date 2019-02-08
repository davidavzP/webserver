use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::sync::MutexGuard;


fn main() {
    loop {
        let listener = TcpListener::bind("10.253.193.212:8888").unwrap();
        let total_requests = Arc::new(Mutex::new(0));
        let total_invalid_requests = Arc::new(Mutex::new(0));

        for stream in listener.incoming() {
            let total_requests = total_requests.clone();
            let total_invalid_requests = total_invalid_requests.clone();
            create_connection(stream.unwrap(), total_requests, total_invalid_requests);
        }
    }

}

fn create_connection(stream: TcpStream, total_requests: Arc<Mutex<i32>>, total_invalid_requests: Arc<Mutex<i32>>){
    let stream = wrap_stream(stream);
    thread::spawn(move || {
        let stream = stream.lock().unwrap();
        let total_requests_num = increase_total_requests(total_requests);
        let total_invalid_requests = total_invalid_requests.lock().unwrap();
        handle_connection(stream, total_requests_num, total_invalid_requests);
    });

}

fn wrap_stream(stream: TcpStream) -> Arc<Mutex<TcpStream>>{
    let stream = Arc::new(Mutex::new(stream));
    let stream = stream.clone();
    stream
}

fn increase_total_requests(total_requests: Arc<Mutex<i32>>) -> i32{
    let mut total_requests = total_requests.lock().unwrap();
    *total_requests += 1;
    total_requests.clone()
}

fn handle_connection(mut stream: MutexGuard<TcpStream>, total_requests_num: i32, mut total_invalid_requests: MutexGuard<i32>) {

        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        let address = stream.peer_addr().unwrap();

        let message = String::from_utf8_lossy(&buffer[..]).to_string();
        println!("{}", message);
        let message: Vec<&str> = message.split_whitespace().collect();

        let request_file = message.get(1);
        let request_file = request_file.unwrap();

        let get = b"GET / HTTP/1.1\r\n";

        if buffer.starts_with(get) {
            let response = format!("HTTP/1.1 200 OK
            Content-Type: text/html; charset=UTF-8

            <html>
            <body>
            <h1>Request Valid</h1>
            Client address: {}<br>
            Requested file: {}<br>
            Total Requests: {}<br>
            Total Invalid Requests: {}<br>
            </body>
            </html>", address, request_file, total_requests_num, *total_invalid_requests);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }else {
            *total_invalid_requests += 1;
            let response = format!("HTTP/1.1 403 Forbidden
            Content-Type: text/html; charset=UTF-8

                <html>
                <body>
                <h1>Request not Valid</h1>
                Client address: {}<br>
                Requested file: {}<br>
                Total Requests: {}<br>
                Total Invalid Requests: {}<br>
                </body>
                </html>", address, request_file, total_requests_num, *total_invalid_requests);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();

        }
}






