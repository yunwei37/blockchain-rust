#![cfg(feature = "bitcoin_hashes")]
#![cfg(feature = "rand")]

extern crate bitcoin_hashes;
extern crate rand;
extern crate secp256k1;

use super::*;
use bincode::{deserialize, serialize};
use bitcoin_hashes::hash160::Hash as Hash160;
use bitcoin_hashes::sha256;
use bitcoincash_addr::Address;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use rand::rngs::OsRng;
use secp256k1::{key, Message, Secp256k1};
use sled;
use std::collections::HashMap;

const VERSION: u8 = 0;
const addressChecksumLen: usize = 4;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    secret_key: key::SecretKey,
    public_key: key::PublicKey,
}

impl Wallet {
    fn new() -> Self {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);
        Wallet {
            secret_key,
            public_key,
        }
    }

    fn get_address(&self) -> Vec<u8> {
        let pubkeyhash = Hash160::hash(self.public_key).to_vec();
        let address = Address {
            body: pubkeyhash,
            ..Default::default()
        };
        address.encode().unwrap()
    }
}

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    /// NewWallets creates Wallets and fills it from a file if it exists
    pub fn new() -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };
        let db = sled::open("data/wallets")?;

        for i in db.into_iter()? {
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet = deserialize(i.1.to_vec())?;
            wlt.wallets.insert(address, wallet);
        }
        OK(wlt)
    }

    /// CreateWallet adds a Wallet to Wallets
    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        info!("create wallet: {}", address);
        address
    }

    /// GetAddresses returns an array of addresses stored in the wallet file
    pub fn get_all_addresses(&self) -> Vec<String> {
        let addresses = Vec::<String>::new();
        for (address, _) in self.wallets {
            addresses.push(address);
        }
        addresses
    }

    /// GetWallet returns a Wallet by its address
    pub fn get_wallet(&self, address: &str) -> Option<Wallet> {
        self.wallets.get(address)
    }

    /// SaveToFile saves wallets to a file
    pub fn save_all(&self) -> Result<()> {
        let db = sled::open("data/wallets")?;

        for (address, wallet) in self.wallets {
            let data = serialize(wallet)?;
            db.insert(wallet.get_address(), data)?;
        }

        db.flush()?;
        Ok(())
    }
}

fn test() {
    let message = Message::from_hashed_data("Hello World!".as_bytes());

    let sig = secp.sign(&message, &secret_key);
    assert!(secp.verify(&message, &sig, &public_key).is_ok());
}
