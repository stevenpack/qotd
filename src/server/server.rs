extern crate mio;
extern crate rand;

use mio::{EventLoop,Token,TryWrite, Interest, PollOpt};
use mio::tcp::*;
use mio::udp::*;
use mio::buf::*;
use rand::{thread_rng, Rng};
use std::fs::File;
use std::io::*;

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
                            quotes.push(line_str + "\n");
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

struct QotdServer {
    tcp_server: TcpListener,
    udp_server: UdpSocket,
    quote_provider: QuoteProvider
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

fn main() {
    
    let quote_provider: QuoteProvider = QuoteProviderImpl::new();
    println!("Sample quote: {:?}", quote_provider.get_random_quote());

    let address = "0.0.0.0:6567".parse().unwrap();
    let tcp_server = TcpListener::bind(&address).unwrap();
    let udp_server = UdpSocket::v4().unwrap();
    let _ = udp_server.bind(&address);

    let mut event_loop = mio::EventLoop::new().unwrap();
    let _ = event_loop.register_opt(&tcp_server, TCP_SERVER, Interest::readable(), PollOpt::edge());
    let _ = event_loop.register_opt(&udp_server, UDP_SERVER, Interest::readable(), PollOpt::edge());

    println!("running qotd server");
    let mut qotd_server = QotdServer 
    {
        tcp_server: tcp_server,
        udp_server: udp_server,
        quote_provider: quote_provider        
    };
    let _ = event_loop.run(&mut qotd_server);

    drop(qotd_server.udp_server);
    drop(qotd_server.tcp_server);

}