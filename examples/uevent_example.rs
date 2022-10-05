extern crate libc;
extern crate mio;
extern crate netlink_rust;

use std::time::Duration;

use netlink_rust as netlink;

use crate::netlink::{Protocol, Socket};
use std::collections::HashMap;

use mio::unix::EventedFd;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::os::unix::io::AsRawFd;

fn parse_uevent(message: &str) -> HashMap<String, String> {
    let mut arguments = HashMap::new();
    let mut msg_iter = message.split('\0');
    msg_iter.next(); // skip the first value
    for arg in msg_iter {
        let key_val: Vec<&str> = arg.splitn(2, '=').collect();
        if key_val.len() == 2 {
            arguments.insert(String::from(key_val[0]), String::from(key_val[1]));
        }
    }
    return arguments;
}

fn receive_messages(socket: &mut Socket) {
    loop {
        let result = socket.receive();
        match result {
            Err(e) => {
                println!("Failed to receive message(s), {:?}", e);
                break;
            }
            Ok(data) => {
                if data.is_empty() {
                    break;
                }
                let text = String::from_utf8(data);
                match text {
                    Ok(text) => {
                        println!("Event --------");
                        let args = parse_uevent(&text);
                        for (key, value) in args {
                            println!("{:16}: {}", key, value);
                        }
                    }
                    Err(_) => println!("Failed to convert bytes to text"),
                }
            }
        }
    }
}

fn main() {
    const NETLINK: Token = Token(1);
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    let timeout = Duration::from_millis(500);
    // When listening to uevents we need to provide the multicast group 1
    let mut socket = Socket::new_multicast(Protocol::KObjectUevent, 1).unwrap();
    // register socket in event loop
    poll.register(
        &EventedFd(&socket.as_raw_fd()),
        NETLINK,
        Ready::readable(),
        PollOpt::edge(),
    )
    .unwrap();
    loop {
        poll.poll(&mut events, Some(timeout)).unwrap();
        for event in events.iter() {
            match event.token() {
                NETLINK => {
                    receive_messages(&mut socket);
                }
                _ => unreachable!(),
            }
        }
    }
}
