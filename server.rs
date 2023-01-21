use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const MSG_SIZE: usize = 1000;

fn server() -> String {
    println!("\nEnter server ip");
    let mut ip = String::new();
    io::stdin().read_line(&mut ip).expect("Reading from stdin failed");
    let server_ip = ip.trim().to_string();

    println!("\nEnter server port");
    let mut port = String::new();
    io::stdin().read_line(&mut port).expect("Reading from stdin failed");
    let server_port = port.trim().to_string();

    let server_address = format!("{}{}{}", &server_ip, &String::from(":"), &server_port);

    server_address
}

fn main() {
    println!("[Server]");

    let server = TcpListener::bind(server()).expect("Lister failed to bind");
    server.set_nonblocking(true).expect("failed to initialize non blocking listener");

    println!("\nWaiting for client connection..");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));

            let mut finalname = "".to_string();
            let mut name_buff = vec![0; MSG_SIZE];
            match socket.read_exact(&mut name_buff) {
                Ok(_) => {
                    let name = name_buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    let name = String::from_utf8(name).expect("invalid utf8 name");
                    let msg = format!("[{}]{}님이 입장하셨습니다.", addr, name);
                    tx.send(msg).expect("failed to send message to rx");
                    finalname = name.clone();
                },
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("Error loading client name");
                    break;
                }
            }

            println!("client [{}]{} connected", addr, finalname);

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("invalid utf8 message");

                        println!("[{}]{}", addr, msg);

                        let msg = format!("[{}]{}님이 {}을(를) 입력하셨습니다.", addr, finalname, &msg);
                        tx.send(msg).expect("failed to send message to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with [{}]{}}", addr, finalname);
                        let msg = format!("[{}]{}님이 퇴장하셨습니다.", addr, finalname);
                        tx.send(msg).expect("failed to send message to rx");
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
            })
                .collect::<Vec<_>>();
        }
        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}