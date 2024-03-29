extern crate netlink_rust;

use netlink_rust as netlink;

use crate::netlink::generic;
use crate::netlink::route;
use crate::netlink::route::{AddressFamilyAttribute, InterfaceInformationMessage};
use crate::netlink::{Message, Protocol, Socket};

fn handle_message(message: &Message) {
    if message.header.identifier == route::FamilyId::NewLink {
        let (_, msg) = InterfaceInformationMessage::unpack(&message.data).unwrap();
        for attr in msg.attributes {
            if attr.identifier == AddressFamilyAttribute::InterfaceName {
                let name = attr.as_string().unwrap();
                println!("{}", name);
            }
        }
    } else {
        println!("Header: {}", message.header);
    }
}

fn get_network_interfaces(socket: &mut Socket) {
    {
        let tx_msg = route::Message::new(route::FamilyId::GetLink);
        socket.send_message(&tx_msg).unwrap();
    }
    let messages = socket.receive_messages().unwrap();
    for message in messages {
        handle_message(&message);
    }
}

fn main() {
    let mut gen_socket = Socket::new(Protocol::Generic).unwrap();
    let mut rt_socket = Socket::new(Protocol::Route).unwrap();

    println!("----------------------------------------------------------------");
    println!("get_network_interfaces");
    println!("----------------------------------------------------------------");
    get_network_interfaces(&mut rt_socket);
    println!("----------------------------------------------------------------");
    println!("get families");
    println!("----------------------------------------------------------------");
    for family in generic::Family::all(&mut gen_socket).unwrap() {
        println!("{}", family);
    }
    println!("----------------------------------------------------------------");
    match generic::Family::from_name(&mut gen_socket, "nl80211") {
        Ok(id) => {
            println!("Found nl80211, {}", id);
        }
        Err(_) => {
            println!("Failed to find nl80211");
        }
    }
    println!("----------------------------------------------------------------");
    match generic::Family::from_name(&mut gen_socket, "HELLO_THERE") {
        Ok(id) => {
            println!("Found HELLO_THERE, {}", id);
        }
        Err(_) => {
            println!("Failed to find HELLO_THERE");
        }
    }
}
