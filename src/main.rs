
extern crate reqwest;

use std::time;
use std::net;
use std::io::{Read, Write};

const APPSPOT: &str = "https://realmofthemadgodhrd.appspot.com/char/list?guid=";

fn main() {
    println!("<<< RotMG Server Ping Checker by Zeroeh >>>");
    println!("Grabbing server list...");
    let mut servers = grab_server_list();
    println!("Checking ping times for {} servers. This should only take about 10 seconds...", servers.len());
    for server in servers.iter_mut() {
        server.check_ping();
    }
    let mut fastest_server = &Server::new(String::new(), String::new());
    for server in servers.iter() {
        println!("{0} -> {1}ms", server.name, server.ping_time);
        if fastest_server.ping_time == 0.0 {
            fastest_server = server;
        } else {
            if fastest_server.ping_time > server.ping_time {
                fastest_server = server; //set the new fastest
            }
        }
    }
    println!("Your fastest server is {0} with a ping of {1}ms", fastest_server.name, fastest_server.ping_time);
}

fn grab_server_list() -> Vec<Server> {
    let full_url = String::from(APPSPOT) + &time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs().to_string();
    match reqwest::get(&full_url) {
        Ok(mut t) => {
            match t.text() {
                Ok(text) => {
                    if text.len() < 100 { //quick check to make sure the response we got is valid
                        panic!("Didn't get a valid response. Got {}", text);
                    }
                    let index1 = text.find("<Servers>").unwrap() + 9;
                    let index2 = text.find("</Servers>").unwrap();
                    let servers = &text[index1..index2].split("<Server>").collect::<Vec<&str>>();
                    let mut server_vec: Vec<Server> = Vec::new();
                    let mut idx = 0;
                    for server in servers {
                        if idx == 0 { //we get a blank string at [0] for some reason
                            idx += 1;
                            continue;
                        }
                        let name_index1 = server.find("<Name>").unwrap() + 6;
                        let name_index2 = server.find("</Name>").unwrap();
                        let server_name = String::from(&server[name_index1..name_index2]);
                        let ip_index1 = server.find("<DNS>").unwrap() + 5;
                        let ip_index2 = server.find("</DNS>").unwrap();
                        let server_ip = String::from(&server[ip_index1..ip_index2]);
                        let new_server = Server::new(server_name, server_ip);
                        server_vec.push(new_server);
                    }
                    return server_vec;
                },
                Err(e) => println!("Error grabbing text: {}", e),
            }
        },
        Err(e) => println!("Error from request: {}", e),
    };
    Vec::new()
}

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub ping_time: f32,
}

impl Server {
    pub fn new(name: String, ip: String) -> Server {
        Server {
            name: name,
            ip: ip,
            ping_time: 0.0,
        }
    }
    pub fn check_ping(&mut self) {
        let dial_server = self.ip.clone() + &":2050";
        match net::TcpStream::connect(dial_server) {
            Ok(mut link) => {
                link.set_nodelay(true).unwrap();
                let timer = time::Instant::now();
                let buffer: [u8; 8] = [0; 8];
                let written = link.write(&buffer).unwrap();
                if written < buffer.len() {
                    panic!("Didn't write all bytes: {}", written);
                }
                let mut r_buffer: [u8; 1] = [0; 1];
                let read = link.read(&mut r_buffer).unwrap();
                if read != 1 {
                    panic!("Didn't read 1 byte, got {}", read);
                }
                self.ping_time = get_time(timer);
            },
            Err(_) => println!("Host {} forcibly closed the conenction", self.ip.clone()),
        };
    }
}

fn get_time(dur: time::Instant) -> f32 {
    dur.elapsed().subsec_millis() as f32
}
