use std::thread::Thread;
use std::io::BufferedReader;
use std::io::File;
use std::rand::{task_rng, Rng};
use std::sync::Arc;
use std::io::timer::sleep;
use std::time::duration::Duration;

trait QuoteProviderImpl {
  fn get_quote(&self, uint) -> String;
  fn get_random_quote(&self) -> String;
  fn new() -> Self;
}

struct QuoteProvider {
  quotes: Vec<String>
}

impl QuoteProviderImpl for QuoteProvider {
  
  fn get_quote(&self, index: uint) -> String {
    self.quotes[index].clone()
  }

  fn get_random_quote(&self) -> String {
    let index: uint = task_rng().gen_range(0, self.quotes.len());
    self.get_quote(index)
  }

  fn new() -> QuoteProvider {

    let mut quotes: Vec<String> = Vec::new();
    println!("Reading wisdom...");
    //todo: check exists
    let path = Path::new("../src/wisdom.txt");
    let mut file = BufferedReader::new(File::open(&path));
    for line in file.lines() {
      quotes.push(line.unwrap());
    }

    QuoteProvider {quotes: quotes}
  }
}

fn main() {
   //arg handling
   //https://gist.github.com/stevenpack/cef7ae4ac9615eda4b73
   
   let quote_provider = QuoteProviderImpl::new();
   
   //UDP and TCP servers will share the one QuoteProvider
   let quote_provider_arc = Arc::new(quote_provider);

   let quote_provider_shared_udp = quote_provider_arc.clone();
   let quote_provider_shared_tcp = quote_provider_arc.clone();

   //todo: understand move
   //todo: properly handle spawn return
   Thread::spawn(move || {start_udp(quote_provider_shared_udp)}).detach();
   Thread::spawn(move || {start_tcp(quote_provider_shared_tcp)}).detach();
      
    //todo: handle signals, run as server
   println!("Server started. Ctrl-C to stop.");

   loop {
     sleep(Duration::seconds(60));
     println!("Still alive...");
   }
}
