use super::*;
use crate::blockchain::*;
use bincode::{deserialize, serialize};
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::*;
use std::thread;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Addr(Vec<String>),
    Version(Versionmsg),
    Tx(Txmsg),
    GetData(GetDatamsg),
    GetBlock(GetBlocksmsg),
    Inv(Invmsg),
    Block(Blockmsg),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Blockmsg {
    AddrFrom: String,
    Block: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetBlocksmsg {
    AddrFrom: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetDatamsg {
    AddrFrom: String,
    r#type: String,
    id: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Invmsg {
    AddrFrom: String,
    r#type: String,
    Items: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Txmsg {
    AddrFrom: String,
    transaction: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Versionmsg {
    AddrFrom: String,
    version: i32,
    best_height: i32,
}

pub struct Server {
    node_address: String,
    known_nodes: Vec<String>,
    bc: Blockchain,
}

const KNOWN_NODE1: &str = "localhost:3000";
const CMD_LEN: usize = 12;
const VERSION: u32 = 1;

impl Server {
    pub fn start_server(port: &str, miner_address: String) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        let server = Server {
            node_address: String::from("localhost:") + port,
            known_nodes: vec![String::from(KNOWN_NODE1)],
            bc: Blockchain::new()?,
        };
        let server = Arc::new(Mutex::new(server));

        for stream in listener.incoming() {
            let stream = stream?;
            let server = Arc::clone(&server);
            thread::spawn(move || handle_connection(stream, server));
        }

        Ok(())
    }
}

fn handle_version(server: Arc<Mutex<Server>>, msg: Versionmsg) -> Result<()> {
    Ok(())
}

fn handle_addr(server: Arc<Mutex<Server>>, msg: Vec<String>) -> Result<()> {
    Ok(())
}

fn handle_block(server: Arc<Mutex<Server>>, msg: Blockmsg) -> Result<()> {
    Ok(())
}

fn handle_inv(server: Arc<Mutex<Server>>, msg: Invmsg) -> Result<()> {
    Ok(())
}

fn handle_get_blocks(server: Arc<Mutex<Server>>, msg: GetBlocksmsg) -> Result<()> {
    Ok(())
}

fn handle_get_data(server: Arc<Mutex<Server>>, msg: GetDatamsg) -> Result<()> {
    Ok(())
}

fn handle_tx(server: Arc<Mutex<Server>>, msg: Txmsg) -> Result<()> {
    Ok(())
}

fn handle_connection(mut stream: TcpStream, server: Arc<Mutex<Server>>) -> Result<()> {
    let mut buffer = Vec::new();
    let count = stream.read_to_end(&mut buffer)?;
    info!("Accept request: length {}", count);

    let cmd = bytes_to_cmd(&buffer)?;

    match cmd {
        Message::Addr(data) => handle_addr(server, data)?,
        Message::Block(data) => handle_block(server, data)?,
        Message::Inv(data) => handle_inv(server, data)?,
        Message::GetBlock(data) => handle_get_blocks(server, data)?,
        Message::GetData(data) => handle_get_data(server, data)?,
        Message::Tx(data) => handle_tx(server, data)?,
        Message::Version(data) => handle_version(server, data)?,
    }

    Ok(())
}

fn bytes_to_cmd(bytes: &[u8]) -> Result<Message> {
    let mut cmd = Vec::new();
    let cmd_bytes = &bytes[..CMD_LEN];
    let data = &bytes[CMD_LEN..];
    for b in cmd_bytes {
        if 0 as u8 != *b {
            cmd.push(*b);
        }
    }
    info!("cmd: {}", String::from_utf8(cmd.clone())?);

    if cmd == "addr".as_bytes() {
        let data: Vec<String> = deserialize(data)?;
        Ok(Message::Addr(data))
    } else if cmd == "block".as_bytes() {
        let data: Blockmsg = deserialize(data)?;
        Ok(Message::Block(data))
    } else if cmd == "inv".as_bytes() {
        let data: Invmsg = deserialize(data)?;
        Ok(Message::Inv(data))
    } else if cmd == "getblocks".as_bytes() {
        let data: GetBlocksmsg = deserialize(data)?;
        Ok(Message::GetBlock(data))
    } else if cmd == "getdata".as_bytes() {
        let data: GetDatamsg = deserialize(data)?;
        Ok(Message::GetData(data))
    } else if cmd == "tx".as_bytes() {
        let data: Txmsg = deserialize(data)?;
        Ok(Message::Tx(data))
    } else if cmd == "version".as_bytes() {
        let data: Versionmsg = deserialize(data)?;
        Ok(Message::Version(data))
    } else {
        Err(format_err!("Unknown command in the server"))
    }
}
