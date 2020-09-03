use super::*;
use crate::blockchain::*;
use crate::wallets::*;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use failure::format_err;
use serde::{Deserialize, Serialize};

const SUBSIDY: i32 = 10;

/// TXInput represents a transaction input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub script_sig: String,
}

/// TXOutput represents a transaction output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub script_pub_key: String,
}

/// Transaction represents a Bitcoin transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    /// NewUTXOTransaction creates a new transaction
    pub fn new_UTXO(from: &str, to: &str, amount: i32, bc: &Blockchain) -> Result<Transaction> {
        info!("new UTXO Transaction from: {} to: {}", from, to);
        let mut vin = Vec::new();
        let acc_v = bc.find_spendable_outputs(from, amount);

        if acc_v.0 < amount {
            error!("Not Enough balance");
            return Err(format_err!(
                "Not Enough balance: current balance {}",
                acc_v.0
            ));
        }

        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(),
                    vout: out,
                    script_sig: String::from(from),
                };
                vin.push(input);
            }
        }

        let mut vout = vec![TXOutput {
            value: amount,
            script_pub_key: String::from(to),
        }];
        if acc_v.0 > amount {
            vout.push(TXOutput {
                value: acc_v.0 - amount,
                script_pub_key: String::from(from),
            })
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout,
        };
        tx.set_id()?;
        Ok(tx)
    }

    /// NewCoinbaseTX creates a new coinbase transaction
    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        info!("new coinbase Transaction to: {}", to);
        if data == String::from("") {
            data += &format!("Reward to '{}'", to);
        }
        let mut tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: String::new(),
                vout: -1,
                script_sig: data,
            }],
            vout: vec![TXOutput {
                value: SUBSIDY,
                script_pub_key: to,
            }],
        };
        tx.set_id()?;
        Ok(tx)
    }

    /// SetID sets ID of a transaction
    fn set_id(&mut self) -> Result<()> {
        let mut hasher = Sha256::new();
        let data = serialize(self)?;
        hasher.input(&data);
        self.id = hasher.result_str();
        Ok(())
    }

    /// IsCoinbase checks whether the transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }
}


impl TXInput {

    /// UsesKey checks whether the address initiated the transaction
    pub fn uses_key(&self, pub_key_hash: &str) -> bool {
        let pubkeyhash = (self.public_key).to_string();
        pubkeyhash == pub_key_hash
    }
}

impl TXOutput {
    /// IsLockedWithKey checks if the output can be used by the owner of the pubkey
    pub fn is_locked_with_key(&self, pub_key_hash:String) -> bool {
        self.pub_key_hash == pub_key_hash
    }
    /// Lock signs the output
    fn lock(&mut self, address:&str) -> Result<()>{
        let pub_key_hash = Address::decode(address)?.body;
        self.pub_key_hash = String::from_utf8(pub_key_hash)?;
        Ok(())
    }
    
    pub fn new(value:i32, address:String) -> Result<Self> {
        let mut txo = TXOutput{
            value,
            pub_key_hash: String::new(),
        };
        txo.lock(&address);
        Ok(txo)
    }
}
