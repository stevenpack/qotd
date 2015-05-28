extern crate mio;
extern crate std;

use mio::{Token,TryWrite};
use mio::tcp::*;
use mio::udp::*;
use mio::buf::*;
use std::io::*;
use quote_provider::*;

const TCP_SERVER: mio::Token = mio::Token(0);
const UDP_SERVER: mio::Token = mio::Token(1);

pub struct QotdServer {
    pub tcp_server: TcpListener,
    pub udp_server: UdpSocket,
    pub quote_provider: QuoteProvider
}

impl QotdServer {

    fn accept_tcp_connection(quote_provider: &QuoteProvider, tcp_listener: &TcpListener) {
        
        let stream_result = tcp_listener.accept();
        match stream_result {
            Ok(Some(mut connection)) => {
                println!("accepted a socket {:?}", connection);
                let result = connection.write_slice(quote_provider.get_random_quote().as_bytes()) ;
                QotdServer::log_socket_write(result);
                drop(connection);
            }
            Ok(None) => println!("The tcp socket wasn't actually ready"),        
            Err(e) => println!("listener.accept() errored: {:?}", e)            
        }
    }

    fn accept_udp_connection(quote_provider: &QuoteProvider, udp_socket: &UdpSocket) {
        println!("the server socket is ready to accept a UDP connection");
        let mut buf = [0; 128];                
        match udp_socket.recv_from(&mut MutSliceBuf::wrap(&mut buf)) {
            Ok(Some(addr)) => {
                let quote = quote_provider.get_random_quote();
                let mut quote_buf = SliceBuf::wrap(&mut quote.as_bytes());
                let result = udp_socket.send_to(&mut quote_buf, &addr);
                QotdServer::log_socket_write(result);
            }
            Ok(None) => println!("The udp socket wasn't actually ready"),
            Err(e) => println!("couldn't receive a datagram: {}", e)
        }
    }

    fn log_socket_write<T>(result: Result<Option<T>>) where T: std::fmt::Debug {
        match result {
            Ok(Some(bytes)) => println!("Wrote {:?} bytes to socket ", bytes),
            Ok(None) => println!("Didn't write any bytes"),
            Err(e) => println!("Failed to write. {:?}", e)
        }
    }
}

impl mio::Handler for QotdServer {
    type Timeout = ();
    type Message = ();

    #[allow(unused_variables)]
    fn readable(&mut self, event_loop: &mut mio::EventLoop<QotdServer>, token: mio::Token, hint: mio::ReadHint) {
        match token {
            TCP_SERVER => QotdServer::accept_tcp_connection(&self.quote_provider, &self.tcp_server),              
            UDP_SERVER => QotdServer::accept_udp_connection(&self.quote_provider, &self.udp_server),
            _ => panic!("Received unknown token"),
        }
    }
}