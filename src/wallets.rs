use super::*;
use bincode::{deserialize, serialize};
use bitcoincash_addr::Address;
use crypto::ed25519;
use crypto::ripemd160::Ripemd160;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use sled;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const VERSION: u8 = 0;
const addressChecksumLen: usize = 4;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
}

impl Wallet {
    fn new() -> Self {
        let (secret_key, public_key) = ed25519::keypair(&vec![0]);
        let secret_key = secret_key.to_vec();
        let public_key = public_key.to_vec();
        Wallet {
            secret_key,
            public_key,
        }
    }

    fn get_address(&self) -> String {
        let mut pub_hash:Vec<u8> = self.public_key.clone();
        hash_pub_key(&mut pub_hash);
        let address = Address {
            body: pub_hash,
            ..Default::default()
        };
        address.encode().unwrap()
    }
}

pub fn hash_pub_key(pubKey: &mut Vec<u8>){
    let mut hasher1 = Sha256::new();
    hasher1.input(pubKey);
    hasher1.result(pubKey);
    let mut hasher2 = Ripemd160::new();
    hasher2.input(pubKey);
    hasher2.result(pubKey);
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

        for item in db.into_iter() {
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet = deserialize(&i.1.to_vec())?;
            wlt.wallets.insert(address, wallet);
        }
        Ok(wlt)
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
        let mut addresses = Vec::<String>::new();
        for (address, _) in &self.wallets {
            addresses.push(address.clone());
        }
        addresses
    }

    /// GetWallet returns a Wallet by its address
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    /// SaveToFile saves wallets to a file
    pub fn save_all(&self) -> Result<()> {
        let db = sled::open("data/wallets")?;

        for (address, wallet) in &self.wallets {
            let data = serialize(wallet)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        Ok(())
    }
}