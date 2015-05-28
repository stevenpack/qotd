extern crate mio;
extern crate rand;

mod quote_provider;
mod server;

use mio::{EventLoop,Token, Interest, PollOpt};
use mio::tcp::*;
use mio::udp::*;
use quote_provider::*;

pub const TCP_SERVER: mio::Token = mio::Token(0);
pub const UDP_SERVER: mio::Token = mio::Token(1);

#[macro_export]
macro_rules! mytry {
    ($e:expr) => ({
        use ::std::result::Result::{Ok, Err};
 
        match $e {
            Ok(e) => e,
            Err(e) => return Err(e),
        }
    })
}

fn main() {
    println!("QOTD starting. Initializing quote provider.");
    let quote_provider: QuoteProvider = QuoteProviderImpl::new();
    println!("Sample quote: {:?}", quote_provider.get_random_quote());
    println!("Binding sockets");

    let address = "0.0.0.0:6567".parse().unwrap();
    let tcp_server = TcpListener::bind(&address).unwrap();
    let udp_server = UdpSocket::v4().unwrap();
    let _ = udp_server.bind(&address);

    println!("Setting up async IO");
    let mut event_loop = EventLoop::new().unwrap();
    let _ = event_loop.register_opt(&tcp_server, TCP_SERVER, Interest::readable(), PollOpt::edge());
    let _ = event_loop.register_opt(&udp_server, UDP_SERVER, Interest::readable(), PollOpt::edge());

    println!("Starting server");
    let mut qotd_server = server::QotdServer 
    {
        tcp_server: tcp_server,
        udp_server: udp_server,
        quote_provider: quote_provider        
    };
    let _ = event_loop.run(&mut qotd_server);

    drop(qotd_server.udp_server);
    drop(qotd_server.tcp_server);
}