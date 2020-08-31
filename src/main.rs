mod block;
mod blockchain;

pub type Result<T> = std::result::Result<T, failure::Error>;

use blockchain::*;

fn main() -> Result<()> {
    let mut bc = Blockchain::new();
    bc.add_block(String::from("Send 1 BTC to Ivan"))?;
    bc.add_block(String::from("Send 1 BTC to Ivan"))?;

    println!("Blockchain: {:#?}", bc);
    Ok(())
}
