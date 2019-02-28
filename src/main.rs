use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
extern crate regex;
use regex::Regex;
use std::collections::HashMap;
fn main() -> io::Result<()> {
    let cache: Arc<Mutex<HashMap<String,Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
    let count = Arc::new(Mutex::new(0));
    let success = Arc::new(Mutex::new(0));
    let listener = TcpListener::bind("127.0.0.1:8888")?;
    loop {
        for stream in listener.accept() {
            let cache = cache.clone();
            let count = count.clone();
            let success = success.clone();
            thread::spawn(move || {
                acceptAndRespond(stream, count, success, cache);
            //    println!("Spawned a thread!");
            });
        }
    }

    Ok(())
}

fn acceptAndRespond(stream: (TcpStream, SocketAddr), count: Arc<Mutex<i32>>, success: Arc<Mutex<i32>>, cache: Arc<Mutex<HashMap<String,Vec<u8>>>>) {
    let mut count = count.lock().unwrap();

    let mut mutStream = stream.0;
    let socketAddr = stream.1;
    let getRequest = get_GET_request(&mut mutStream);
    let actualRequest = betweenGetHTTP(&getRequest);
    // println!("Actual request: {}", &actualRequest);
    // println!("Our get request is {}", &actualRequest);
    // let requestedPath = getPathFromGET(&getRequest);
    let multiRequest = getMultiplePaths(&actualRequest);
    // println!("{:?}", multiRequest);
    for requestedPath in multiRequest{
        // println!("Currently requested page is : {}", requestedPath);
        if requestedPath.contains("../") {
            let mut success = success.lock().unwrap();
            let returnMe: String = "HTTP/1.1 403 Forbidden\nCount:".to_string() + &count.to_string() + "\nSuccessful: " + &success.to_string() + "\n\n<html><body><h1>Error 403</h1></body></html>";
            mutStream.write(returnMe.as_bytes());
            return;
    } else {
        // println!("Opening {}", &requestedPath);
        let responseToSend = openFileFromPath(&requestedPath, &count, &success, &cache);
        let responseAsBytes = responseToSend.0.as_bytes();
        mutStream.write(responseAsBytes);
        // println!("{:?}",&responseToSend.1);
        mutStream.write(&responseToSend.1);

//        println!("Serving {}", requestedPath);
    }
    let mut success = success.lock().unwrap();
        *count += 1;
    println!("Total number of requests: {}. Proper requests: {}", *count, *success);
    }
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

fn openFileFromPath(path: &String, completed: &MutexGuard<i32>, success: &Arc<Mutex<i32>>, cache: &Arc<Mutex<HashMap<String,Vec<u8>>>>) -> (String,Vec<u8>) {

    let mutPath = &path;
    let mut relPath = "www".to_string() + mutPath;
    let mut cache = cache.lock().unwrap();
    println!("{:?}", cache.keys());
    if !cache.contains_key(&relPath){
        println!("Insert into cache");
            let mut file = File::open(&relPath);
            match file {
            Ok(e) => {
                let mut success = success.lock().unwrap();
                *success += 1;
                let mut mutE = e;
                let mut byteBuff = Vec::new();
                let mut buf: String = "HTTP/1.1 200 OK\nCount:".to_string() + &completed.to_string() + "\nSuccessful:" + &success.to_string() + "\n\n";
                let asVec = File::read_to_end(&mut mutE, &mut byteBuff);
                // println!("{:?}", byteBuff);
                cache.insert(relPath.clone(), byteBuff);
                return (buf, cache.get(&relPath).unwrap().to_vec());
                // return (buf,byteBuff);
            }
            Err(e) => {
                let mut success = success.lock().unwrap();
                return ("HTTP/1.1 404 Not Found\nCount:".to_string() + &completed.to_string() + "\nSuccessful:" + &success.to_string() + "\n\n<html><body><h1>Error 404 </h1></body></html>", Vec::new());
            }
        }
    }else{
        println!("Should read from cache");
        let filePathFromCache = cache.get(&relPath).expect("Couldn't get from cache");
        let mut file = filePathFromCache;
        let mut success = success.lock().unwrap();
        let mut buf: String = "HTTP/1.1 200 OK\nCount:".to_string() + &completed.to_string() + "\nSuccessful:" + &success.to_string() + "\n\n";
        return(buf, cache.get(&relPath).unwrap().to_vec());
    }

}
fn getMultiplePaths(getRequest: &String) -> Vec<String> {
    // println!("{}", getRequest);
    let mut splitted: Vec<&str> = getRequest.split("\n").collect();
    let mut output:Vec<String> = Vec::new();
    // splitted.remove(0);
    // let mut index = splitted[1];
    for mut path in splitted{
        if (path == "/") {
            path = "/index.html";
        }
        output.push(String::from(path));
    }
    return output;
}

fn betweenGetHTTP(line: &str) -> String{
    let withoutGet: String = line.splitn(2, "GET").collect();
    let mut withoutHTTP: String = withoutGet.splitn(2, "HTTP/1.1").collect();
    withoutHTTP = withoutHTTP.trim().to_string();
    let out:Vec<&str> = withoutHTTP.split(" ").collect();
    return String::from(out[0]);
    // let re = Regex::new("GET(.*?)HTTP/1.1").unwrap();
    // let mat = re.find(line).unwrap();
    // println!("{}", mat.start());
    // return 
}