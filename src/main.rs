use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

fn main() -> io::Result<()> {
    let count = Arc::new(Mutex::new(0));
    let success = Arc::new(Mutex::new(0));

    let listener = TcpListener::bind("127.0.0.1:8888")?;
    loop {
        for stream in listener.accept() {
            let count = count.clone();
            let success = success.clone();
            thread::spawn(move || {
                acceptAndRespond(stream, count, success);
//                println!("Spawned a thread!");
            });
        }
    }

    Ok(())
}

fn acceptAndRespond(stream: (TcpStream, SocketAddr), count: Arc<Mutex<i32>>, success: Arc<Mutex<i32>>) {
    let mut count = count.lock().unwrap();
    *count += 1;

    let mut mutStream = stream.0;
    let socketAddr = stream.1;
    let getRequest = get_GET_request(&mut mutStream);
    let requestedPath = getPathFromGET(&getRequest);

    if requestedPath.contains("../") {
        let mut success = success.lock().unwrap();
        let returnMe: String = "HTTP/1.1 403 Forbidden\nCount:".to_string() + &count.to_string() + "\nSuccessful: " + &success.to_string() + "\n\n<html><body><h1>Error 403</h1></body></html>";
        mutStream.write(returnMe.as_bytes());
//        println!("Total number of requests: {}. Proper requests: {}", *count, *success);
        return;
    } else {
        let responseToSend = openFileFromPath(&requestedPath, &count, &success);
        let responseAsBytes = responseToSend.as_bytes();
        mutStream.write(responseAsBytes);
//        println!("Serving {}", requestedPath);
    }
    let mut success = success.lock().unwrap();
    println!("Total number of requests: {}. Proper requests: {}", *count, *success);
}

fn get_GET_request(mutStream: &mut TcpStream) -> String {
    let mut output = String::new();
    let mut buffer = [0; 500];
    mutStream.read(&mut buffer);
    let asString = str::from_utf8(&mut buffer).unwrap().to_string();
    return asString;
}

fn getPathFromGET(getRequest: &String) -> String {
    let splitted: Vec<&str> = getRequest.split(" ").collect();
    let mut index = splitted[1];
    if (index == "/") {
        index = "/index.html";
    }
    return index.to_string();
}

fn openFileFromPath(path: &String, completed: &MutexGuard<i32>, success: &Arc<Mutex<i32>>) -> String {
    let mutPath = &path;
    let mut relPath = "www".to_string() + mutPath;
    let mut file = File::open(relPath);
    match file {
        Ok(e) => {
            let mut success = success.lock().unwrap();
            *success += 1;
            let mut mutE = e;
            let mut buf: String = "HTTP/1.1 200 OK\nCount:".to_string() + &completed.to_string() + "\nSuccessful:" + &success.to_string() + "\n\n";
            let asVec = File::read_to_string(&mut mutE, &mut buf);
            return buf;
        }
        Err(e) => {
            let mut success = success.lock().unwrap();
            return "HTTP/1.1 404 Not Found\nCount:".to_string() + &completed.to_string() + "\nSuccessful:" + &success.to_string() + "\n\n<html><body><h1>Error 404 </h1></body></html>";
        }
    }
}
