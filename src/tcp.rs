use std::io::prelude::*;
use std::io;

pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab
}

pub struct Connection {
    state: State,    
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            state: State::Listen
        }
    }
}

struct SendSequenceSpace {
    // send unacknowledged
    una: usize,
    // send next
    nxt: usize,
    // send window
    wnd: usize,
    // send urgent pointer
    up: bool,
    // segment sequence number used for last window update
    wl1: usize,
    // segment acknowledgement number used for last window update
    wl2: usize,
    // initial send sequence number
    iss: usize,
}

struct RecvSequenceSpace {
    // receive next
    nxt: usize,
    // receive window
    wind: usize,
    // receive urgent pointer
    up: bool,
    // initial receive sequence number
    irs: usize
}

impl Connection {
    pub fn on_packet<'a> (
            &mut self,
            nic: &mut tun_tap::Iface,
            iph: etherparse::Ipv4HeaderSlice<'a>,
            tcph: etherparse::TcpHeaderSlice<'a>,
            data: &'a [u8]
     ) -> io::Result<usize> {
        let mut buf = [0u8; 1500];
        let mut unwritten = &mut buf[..];
        match *self.state {
            State::Closed => {
                return Ok(0);
            }
            State::Listen => {
                if !tcph.syn() {
                    // only expected syn packet
                    return Ok(0);
                }
                // need to establish a connection
                let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), tcph.source_port(), unimplemented!(), unimplemented!());
                syn_ack.syn = true;
                syn_ack.ack = true;
                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    etherparse::IpTrafficClass::Tcp,
                    [
                        iph.destination()[0],
                        iph.destination()[1],
                        iph.destination()[2],
                        iph.destination()[3],
                    ],
                    [
                        iph.source()[0],
                        iph.source()[1],
                        iph.source()[2],
                        iph.source()[3],
                    ],
                );

                // write out headers
                let unwritten = {
                    let mut unwritten = &mut &buf[..];
                    ip.write(unwritten);
                    syn_ack.write(unwritten);
                    unwritten.len();
                };
                nic.send(&buf[..unwritten])
            }
        }
    }
}
