extern crate libc;
extern crate nix;

use libc::c_int;
use nix::{Errno, Error};
use nix::sys::socket::{AddressFamily, SockAddr, SockFlag, SockType};
use nix::sys::socket;

#[repr(C)]
#[derive(Debug)]
pub enum NetlinkSockOpt {
    AddMembership = 1,
    DropMembership = 2,
    PktInfo = 3,
    BroadcastError = 4,
    NoEnobufs = 5,
}

fn setsockopt_int(fd: c_int, level: c_int, option: c_int, val: c_int) -> Result<(), nix::Error> {
    use std::mem;
    let res = unsafe {
        libc::setsockopt(
            fd,
            level,
            option as c_int,
            mem::transmute(&val),
            mem::size_of::<c_int>() as u32,
        )
    };

    if res == -1 {
        return Err(nix::Error::last());
    }

    Ok(())
}

struct NetlinkSocket(c_int);

impl NetlinkSocket {
    pub fn new() -> Result<Self, nix::Error> {
        let sock = socket::socket(
            AddressFamily::Netlink,
            SockType::Raw,
            SockFlag::empty(),
            libc::NETLINK_USERSOCK,
        )?;

        let pid = unsafe { libc::getpid() };
        socket::bind(sock, &SockAddr::new_netlink(pid as u32, 0))?;
        setsockopt_int(sock, 270, libc::NETLINK_ADD_MEMBERSHIP, 22)?;

        Ok(NetlinkSocket(sock))
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<(), nix::Error> {
        let mut cmsg = nix::sys::socket::CmsgSpace::<()>::new();
        socket::recvmsg(
            self.0,
            &[nix::sys::uio::IoVec::from_mut_slice(buf)],
            Some(&mut cmsg),
            nix::sys::socket::MsgFlags::empty(),
        )?;
        self.send(buf).map(|_| ())
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<usize, nix::Error> {
        socket::sendmsg(
            self.0,
            &[nix::sys::uio::IoVec::from_slice(buf)],
            &[],
            nix::sys::socket::MsgFlags::empty(),
            None,
        )
    }
}

fn main() {
    let mut sk = match NetlinkSocket::new() {
        Ok(sock) => {
            println!("sock {:?}", sock.0);
            sock
        }
        Err(Error::Sys(Errno::EPERM)) => {
            println!("Please run as root.");
            return;
        }
        Err(err) => {
            println!("error {}", err);
            return;
        }
    };

    let mut count = 0;
    let mut buf = [0u8; 1024];
    loop {
        println!("[{}]", count);
        if let Err(e) = sk.recv(&mut buf[..]) {
            println!("error {}", e);
            return;
        }
        count += 1;
    }
}
