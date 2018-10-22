use std::mem::size_of;
use std::io;
use std::os::unix::io::{RawFd, AsRawFd};

use libc;

use errors::{Result, NetlinkError, NetlinkErrorKind};

use core::Protocol;
use core::system;
use core::pack::{NativePack, NativeUnpack};
use core::message::{MessageFlags, Header, ErrorMessage, DataMessage,
    Message, netlink_align};

pub trait Sendable {
    fn pack(&self, data: &mut [u8]) -> Result<usize>;
    fn message_type(&self) -> u16;
    fn query_flags(&self) -> MessageFlags;
}

const NLMSG_NOOP: u16 = 1;
const NLMSG_ERROR: u16 = 2;
const NLMSG_DONE: u16 = 3;
// const NLMSG_OVERRUN: u16 = 4;

const NETLINK_ADD_MEMBERSHIP: i32 = 1;
// const NETLINK_DROP_MEMBERSHIP: i32 = 2;
// const NETLINK_PKTINFO: i32 = 3;
// const NETLINK_BROADCAST_ERROR: i32 = 4;
// const NETLINK_NO_ENOBUFS: i32 = 5;
// const NETLINK_RX_RING: i32 = 6;
// const NETLINK_TX_RING: i32 = 7;
// const NETLINK_LISTEN_ALL_NSID: i32 = 8;
// const NETLINK_LIST_MEMBERSHIPS: i32 = 9;
// const NETLINK_CAP_ACK: i32 = 10;
// const NETLINK_EXT_ACK: i32 = 11;

/// Netlink Socket can be used to communicate with the Linux kernel using the
/// netlink protocol.
pub struct Socket {
    local: system::Address,
    peer: system::Address,
    socket: RawFd,
    sequence_next: u32,
    sequence_expected: u32,
    page_size: usize,
    receive_buffer: Vec<u8>,
    send_buffer: Vec<u8>,
    acknowledge_expected: bool,
}

impl Socket {
    /// Create a new Socket
    pub fn new(protocol: Protocol) -> Result<Socket>
    {
        Socket::new_multicast(protocol, 0)
    }

    /// Create a new Socket which subscribes to the provided multi-cast groups
    pub fn new_multicast(protocol: Protocol, groups: u32) -> Result<Socket>
    {
        let socket = system::netlink_socket(protocol as i32)?;
        system::set_socket_option(socket,
            libc::SOL_SOCKET, libc::SO_SNDBUF, 32768)?;
        system::set_socket_option(socket,
            libc::SOL_SOCKET, libc::SO_RCVBUF, 32768)?;
        let mut local_addr = system::Address {
            family: libc::AF_NETLINK as u16,
            _pad: 0,
            pid: 0,
            groups: groups,
        };
        system::bind(socket, &mut local_addr)?;
        system::get_socket_address(socket, &mut local_addr)?;
        let page_size = netlink_align(system::get_page_size());
        let peer_addr = system::Address {
            family: libc::AF_NETLINK as u16,
            _pad: 0,
            pid: 0,
            groups: groups,
        };
        Ok(Socket {
            local: local_addr,
            peer: peer_addr,
            socket: socket,
            sequence_next: 1,
            sequence_expected: 0,
            page_size: page_size,
            receive_buffer: vec![0u8; page_size],
            send_buffer: vec![0u8; page_size],
            acknowledge_expected: false,
        })
    }

    /// Subscribe to the multi-cast group provided
    pub fn multicast_group_subscribe(&mut self, group: u32) -> Result<()>
    {
        system::set_socket_option(self.socket, libc::SOL_NETLINK,
            NETLINK_ADD_MEMBERSHIP as i32, group as i32)?;
        Ok(())
    }

    #[cfg(not(target_env = "musl"))]
    fn message_header(&mut self, iov: &mut [libc::iovec]) -> libc::msghdr
    {
        let addr_ptr = &mut self.peer as *mut system::Address;
        libc::msghdr {
            msg_iovlen: iov.len(),
            msg_iov: iov.as_mut_ptr(),
            msg_namelen: size_of::<system::Address>() as u32,
            msg_name: addr_ptr as *mut libc::c_void,
            msg_flags: 0,
            msg_controllen: 0,
            msg_control: 0 as *mut libc::c_void,
        }
    }

    #[cfg(target_env = "musl")]
    fn message_header(&mut self, iov: &mut [libc::iovec]) -> libc::msghdr
    {
        let addr_ptr = &mut self.peer as *mut Address;
        libc::msghdr {
            msg_iovlen: iov.len() as i32,
            msg_iov: iov.as_mut_ptr(),
            msg_namelen: size_of::<system::Address>() as u32,
            msg_name: addr_ptr as *mut libc::c_void,
            msg_flags: 0,
            msg_controllen: 0,
            msg_control: 0 as *mut libc::c_void,
        }
    }

    /// Send the provided package on the socket
    pub fn send_message<S: Sendable>(&mut self, payload: &S) -> Result<usize>
    {
        let hdr_size = size_of::<Header>();
        let flags = payload.query_flags();
        let payload_size = payload.pack(&mut self.send_buffer[hdr_size..])?;
        let size = hdr_size + payload_size;
        let hdr = Header {
            length: size as u32,
            identifier: payload.message_type(),
            flags: flags.bits(),
            sequence: self.sequence_next,
            pid: self.local.pid,
        };
        {
            let _slice = hdr.pack(&mut self.send_buffer[..hdr_size])?;
        }

        let mut iov = [
            libc::iovec {
                iov_base: self.send_buffer.as_mut_ptr() as *mut libc::c_void,
                iov_len: size,
            },
        ];

        let msg_header = self.message_header(&mut iov);

        self.acknowledge_expected = flags.contains(MessageFlags::ACKNOWLEDGE);
        self.sequence_expected = self.sequence_next;
        self.sequence_next += 1;

        Ok(system::send_message(self.socket, &msg_header, 0)?)
    }

    fn receive_bytes(&mut self) -> Result<usize>
    {
        let mut iov = [
            libc::iovec {
                iov_base: self.receive_buffer.as_mut_ptr()
                    as *mut libc::c_void,
                iov_len: self.page_size,
            },
        ];
        let mut msg_header = self.message_header(&mut iov);
        let result = system::receive_message(self.socket, &mut msg_header);
        match result {
            Err(err) => {
                if err.raw_os_error() == Some(libc::EAGAIN) {
                    return Ok(0);
                }
                Err(err.into())
            }
            Ok(bytes) => {
                Ok(bytes)
            }
        }
    }

    /// Receive binary data on the socket
    pub fn receive(&mut self) -> Result<Vec<u8>>
    {
        let bytes = self.receive_bytes()?;
        Ok(self.receive_buffer[0..bytes].to_vec())
    }

    /// Receive Messages pending on the socket
    pub fn receive_messages(&mut self) -> Result<Vec<Message>>
    {
        let mut more_messages = true;
        let mut result_messages = Vec::new();
        while more_messages {
            match self.receive_bytes() {
                Err(err) => {
                    return Err(err);
                }
                Ok(bytes) => {
                    if bytes == 0 {
                        break;
                    }
                    more_messages = self.unpack_data(bytes,
                        &mut result_messages)?;
                }
            }
        }
        Ok(result_messages)
    }

    fn unpack_data(&self, bytes: usize, messages: &mut Vec<Message>)
        -> Result<bool>
    {
        let mut more_messages = false;
        let data = &self.receive_buffer[..bytes];
        let mut pos = 0;
        while pos < bytes {
            let (used, header) = Header::unpack_with_size(&data[pos..])?;
            pos = pos + used;
            if !header.check_pid(self.local.pid) {
                return Err(NetlinkError::new(NetlinkErrorKind::InvalidValue)
                    .into());
            }
            if !header.check_sequence(self.sequence_expected) {
                return Err(NetlinkError::new(NetlinkErrorKind::InvalidValue)
                    .into());
            }
            if header.identifier == NLMSG_NOOP {
                continue;
            }
            else if header.identifier == NLMSG_ERROR {
                let (used, emsg) = ErrorMessage::unpack(&data[pos..], header)?;
                pos = pos + used;
                if emsg.code != 0 {
                    return Err(
                        io::Error::from_raw_os_error(-emsg.code).into());
                }
                else {
                    messages.push(Message::Acknowledge);
                }
            }
            else if header.identifier == NLMSG_DONE {
                messages.push(Message::Done);
                pos = pos + header.aligned_data_length();
            }
            else {
                let flags = MessageFlags::from_bits(header.flags)
                    .unwrap_or(MessageFlags::empty());
                let (used, msg) = DataMessage::unpack(&data[pos..], header)?;
                pos = pos + used;
                messages.push(Message::Data(msg));
                if flags.contains(MessageFlags::MULTIPART)
                    || self.acknowledge_expected {
                    more_messages = true;
                }
            }
        }
        return Ok(more_messages);
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd
    {
        self.socket
    }
}
