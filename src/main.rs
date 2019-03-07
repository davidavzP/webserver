use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use std::string::String;
use std::sync::{Arc, Mutex};
use std::sync::MutexGuard;
use std::fs;


fn main() {
    loop {
        match  TcpListener::bind("10.253.193.212:8888"){
            Ok(listener) =>{
                let total_requests = Arc::new(Mutex::new(0));
                let total_invalid_requests = Arc::new(Mutex::new(0));
                let cache:Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

                for stream in listener.incoming() {
                    let total_requests = total_requests.clone();
                    let total_invalid_requests = total_invalid_requests.clone();
                    let cache = cache.clone();
                    create_connection(stream.unwrap(), total_requests, total_invalid_requests, cache);
                }
            }
            Err(e) =>{println!("Error: {}", e); return;

            }
        }

    }

}

fn create_connection(stream: TcpStream,
                     total_requests: Arc<Mutex<i32>>,
                     total_invalid_requests: Arc<Mutex<i32>>,
                     cache: Arc<Mutex<Vec<String>>>){

    //let stream = wrap_mutex(stream);
    thread::spawn(move || {
        //let stream = stream.lock().unwrap();
        let total_requests_num = increase_total_requests(total_requests);

        let total_invalid_requests = total_invalid_requests.lock().unwrap();
        let cache = cache.lock().unwrap();

        match  handle_connection(stream, total_requests_num, total_invalid_requests, cache){
            Ok(()) =>{}
            Err(e) =>{eprintln!("Error: {}", e)}
        }
    });



}

fn wrap_mutex<E>(item: E) -> Arc<Mutex<E>>{
    let item = Arc::new(Mutex::new(item));
    let item = item.clone();
    item
}

fn increase_total_requests(total_requests: Arc<Mutex<i32>>) -> i32{
    let mut total_requests = total_requests.lock().unwrap();
    *total_requests += 1;
    total_requests.clone()
}

fn handle_connection(mut stream: TcpStream, total_requests_num: i32,
                     mut total_invalid_requests: MutexGuard<i32>,
                     cache: MutexGuard<Vec<String>>) -> std::io::Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let address = stream.peer_addr().unwrap();

    let message = String::from_utf8_lossy(&buffer[..]).to_string();
    //println!("{}", message);
    let files = multi_get(message.clone());
    //println!("files:{:?}", files);


    let message: Vec<&str> = message.split_whitespace().collect();

    let request_file = message.get(1);
    if request_file.is_some() {
        let request_file = request_file.unwrap();
        //println!("{}", request_file);
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
        } else {
            let response = Arc::new(Mutex::new(String::new()));

            for mut file in files {

                let response = response.clone();

                let stripped = file.remove(0).to_string();
                eprintln!("file: {}", file);
                    match fs::read_to_string(file) {
                        Ok(contents) => {

                                let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);

                                stream.write(response.as_bytes()).unwrap();
                                stream.flush().unwrap();


                        }
                        Err(e) => {
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
                    };

            }
        }
    }
    Ok(())
}

fn multi_get(message: String) -> Vec<String> {
    let vec: Vec<&str> = message.split_whitespace().collect();
    let mut files: Vec<String> = Vec::new();
    for word in vec {
        if word.starts_with("/") {
            files.push(word.to_string());
        }
    }
    files
}







