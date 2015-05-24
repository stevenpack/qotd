extern crate mio;
extern crate rand;

use mio::tcp::*;
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
            for line in reader.lines {
                match line {
                    Ok (line_str) => {
                        if line_str.len() > 0 {
                            quotes.push(line_str);
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


const SERVER: mio::Token = mio::Token(0);

struct Pong {
    server: TcpListener,
}

impl mio::Handler for Pong {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, event_loop: &mut mio::EventLoop<Pong>, token: mio::Token, hint: mio::ReadHint) {
        match token {
            SERVER => {
                println!("the server socket is ready to accept a connection");
                match self.server.accept() {
                    Ok(Some(connection)) => {
                        println!("accepted a socket, exiting program");
                        event_loop.shutdown();
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
            _ => panic!("Received unknown token"),
        }
    }
}

fn main() {
    
    let quote_provider: QuoteProvider = QuoteProviderImpl::new();
    println!("Sample quote: {:?}", quote_provider.get_random_quote());

    let address = "0.0.0.0:6567".parse().unwrap();
    let server = TcpListener::bind(&address).unwrap();

    let mut event_loop = mio::EventLoop::new().unwrap();
    let _ = event_loop.register(&server, SERVER);

    println!("running pingpong server");
    let _ = event_loop.run(&mut Pong { server: server });
}