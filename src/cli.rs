use super::*;
use crate::blockchain::*;
use clap::{App, Arg};

pub struct Cli {
    bc: Blockchain,
}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {
            bc: Blockchain::new()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        info!("run app");
        let matches = App::new("blockchain-demo")
            .version("0.1")
            .author("yunwei37. 1067852565@qq.com")
            .about("reimplement blockchain_go in rust: a simple blockchain for learning")
            .subcommand(App::new("printchain").about("print all the chain blocks"))
            .subcommand(
                App::new("addblock")
                    .about("add a block in the blockchain")
                    .arg(Arg::from_usage("<data> 'the blockchain data'")),
            )
            .get_matches();

        if let Some(ref matches) = matches.subcommand_matches("addblock") {
            if let Some(c) = matches.value_of("data") {
                self.addblock(String::from(c))?;
            } else {
                println!("Not printing testing lists...");
            }
        }

        if let Some(_) = matches.subcommand_matches("printchain") {
            self.print_chain();
        }

        Ok(())
    }

    fn print_chain(&mut self) {
        for b in self.bc.iter() {
            println!("block: {:#?}", b);
        }
    }

    fn addblock(&mut self, data: String) -> Result<()> {
        self.bc.add_block(data)
    }
}
