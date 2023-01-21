use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const MSG_SIZE: usize = 1000;

fn main() {
    println!("[Client]");

    println!("\nEnter server server_address");
    let mut address = String::new();
    io::stdin().read_line(&mut address).expect("Reading from stdin failed");
    let server_address = address.trim().to_string();

    println!("\nwhat is your name?");
    let mut name_buff = String::new();
    io::stdin().read_line(&mut name_buff).expect("Reading from stdin failed");
    let name = name_buff.trim().to_string();

    let mut client = TcpStream::connect(server_address).expect("stream failed to connect");
    client.set_nonblocking(true).expect("fialed to initialize non-blocking");

    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg_byte_vec = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg_byte_vec).expect("invalid utf8 message");
                println!("----------------------------------------------------------------------------------------------------");
                println!("{}", msg);
                println!("----------------------------------------------------------------------------------------------------");
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was served");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });
    
    let give_name = format!("{}", &name);
    if tx.send(give_name).is_err() { 
        println!("Error sending name to server"); 
    }
    println!("\nPlease enter a message to send..\n\n");

    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Reading from stdin failed");
        let msg = format!("{:?}", &buff.trim().to_string());
        if tx.send(msg).is_err() { break }
    }

}