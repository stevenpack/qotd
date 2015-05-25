extern crate mio;
extern crate rand;

use mio::{EventLoop,Token,TryWrite};
use mio::tcp::*;
use mio::udp::*;
use mio::buf::*;
use rand::{thread_rng, Rng};
use std::fs::File;
use std::io::*;

trait QuoteProviderImpl {
  fn get_quote(&self, usize) -> String;
  fn get_random_quote(&self) -> String;
  fn new() -> Self;
}

struct QuoteProvider {
  quotes: Vec<String>,
}

impl QuoteProviderImpl for QuoteProvider {
  
  fn get_quote(&self, index: usize) -> String {
    //todo: pointers not clones?
    self.quotes[index].clone()
  }

  fn get_random_quote(&self) -> String {
    //todo: cache rng for improved perf
    let mut rng = thread_rng();
    let index: usize = rng.gen_range(0, self.quotes.len());
    self.get_quote(index)
  }

  fn new() -> QuoteProvider {
    let mut quotes: Vec<String> = Vec::new();
    println!("Reading wisdom...");
    //todo: check exists
    //todo: pass in as arg
    let path = "../../src/server/wisdom.txt";
    let result = File::open(path);
    match result {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok (line_str) => {
                        if line_str.len() > 0 {
                            quotes.push(line_str + "\r\n");
                        } else {
                            println!("Skipped blank line in file.");
                        }
                    },
                    Err(e) => println!("Invalid line. Skipping... {:?}",e)
                }
            }
            
        },
        Err(e) => println!("Failed to open wisdom file {:?}. {:?}", path, e)
    }

    QuoteProvider {quotes: quotes}
  }
}

const TCP_SERVER: mio::Token = mio::Token(0);
const UDP_SERVER: mio::Token = mio::Token(1);

struct Pong {
    tcp_server: TcpListener,
    udp_server: UdpSocket,
    quote_provider: QuoteProvider
}

impl mio::Handler for Pong {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, event_loop: &mut mio::EventLoop<Pong>, token: mio::Token, hint: mio::ReadHint) {
        match token {
            TCP_SERVER => {
                println!("the server socket is ready to accept a TCP connection");
                match self.tcp_server.accept() {
                    Ok(Some(mut connection)) => {
                        println!("accepted a socket {:?}", connection);
                        let quote = self.quote_provider.get_random_quote();
                        connection.write_slice(quote.as_bytes());
                    }
                    Ok(None) => {
                        println!("the server socket wasn't actually ready");
                    }
                    Err(e) => {
                        println!("listener.accept() errored: {}", e);
                        event_loop.shutdown();
                    }
                }
            }
            UDP_SERVER => {
                println!("the server socket is ready to accept a UDP connection");
                //let mut buf = [0, ..10];
                match self.udp_server.recv_from(buf) {
                    Ok((amt, src)) => {
                        // Send a reply to the socket we received data from
                        //let buf = buf.mut_slice_to(amt);
                        //buf.reverse();
                        //self.udp_server.send_to(buf, src);
                    }
                    Err(e) => println!("couldn't receive a datagram: {}", e)
                }
            }
            _ => panic!("Received unknown token"),
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<Pong>, token: Token) {
        match token {
            TCP_SERVER => println!("received writable for token 0"),
            _ => panic!("Unexpected token")
        };
    }
}

fn main() {
    
    let quote_provider: QuoteProvider = QuoteProviderImpl::new();
    println!("Sample quote: {:?}", quote_provider.get_random_quote());

    let address = "0.0.0.0:6567".parse().unwrap();
    let tcp_server = TcpListener::bind(&address).unwrap();
    let udp_server = UdpSocket::v4().unwrap();
    udp_server.bind(&address);

    let mut event_loop = mio::EventLoop::new().unwrap();
    let _ = event_loop.register(&tcp_server, TCP_SERVER);

    println!("running pingpong server");
    let _ = event_loop.run(&mut Pong 
    {
        tcp_server: tcp_server,
        udp_server: udp_server,
        quote_provider: quote_provider        
    });

    drop(udp_server); // close the socket
    drop(tcp_server);

}