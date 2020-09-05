use super::*;
use crate::block::*;
use crate::blockchain::*;
use crate::transaction::*;
use bincode::{deserialize, serialize};
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
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
    addr_from: String,
    block: Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetBlocksmsg {
    addr_from: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetDatamsg {
    addr_from: String,
    kind: String,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Invmsg {
    addr_from: String,
    kind: String,
    items: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Txmsg {
    addr_from: String,
    transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Versionmsg {
    addr_from: String,
    version: i32,
    best_height: i32,
}

pub struct Server {
    node_address: String,
    inner: Arc<Mutex<ServerInner>>,
}

struct ServerInner {
    known_nodes: Vec<String>,
    bc: Blockchain,
}

const KNOWN_NODE1: &str = "localhost:3000";
const CMD_LEN: usize = 12;
const VERSION: i32 = 1;

impl Server {
    pub fn start_server(port: &str, miner_address: String) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        let server = Server {
            node_address: String::from("localhost:") + port,
            inner: Arc::new(Mutex::new(ServerInner {
                known_nodes: vec![String::from(KNOWN_NODE1)],
                bc: Blockchain::new()?,
            })),
        };

        for stream in listener.incoming() {
            let stream = stream?;
            let server1 = Server {
                node_address: server.node_address.clone(),
                inner: Arc::clone(&server.inner),
            };
            thread::spawn(move || handle_connection(stream, server1));
        }

        Ok(())
    }

    fn remove_node(&self, addr: &str) {
        let mut nodes = &self.inner.lock().unwrap().known_nodes;
        let mut vec = Vec::new();
        nodes.iter().fold(&mut vec, |vec, x| {
            if x != addr {
                vec.push(x.clone());
            };
            vec
        });
        nodes.clone_from(&&vec);
    }

    fn get_known_nodes(&self) -> Vec<String> {
        self.inner.lock().unwrap().known_nodes.clone()
    }

    fn get_best_height(&self) -> Result<i32> {
        self.inner.lock().unwrap().bc.get_best_height()
    }
}

fn send_data(server: Server, addr: &str, data: &[u8]) -> Result<()> {
    let mut stream = match TcpStream::connect(addr) {
        Ok(s) => s,
        Err(_) => {
            server.remove_node(addr);
            return Ok(());
        }
    };

    stream.write(data)?;

    Ok(())
}

fn send_block(server: Server, addr: &str, b: &Block) -> Result<()> {
    let data = Blockmsg {
        addr_from: server.node_address.clone(),
        block: b.clone(),
    };
    let data = serialize(&(cmd_to_bytes("block"), data))?;
    send_data(server, addr, &data)
}

fn send_addr(server: Server, addr: &str) -> Result<()> {
    let nodes = server.get_known_nodes();
    let data = serialize(&(cmd_to_bytes("addr"), nodes))?;
    send_data(server, addr, &data)
}

fn send_inv(server: Server, addr: &str, kind: &str, items: Vec<Vec<u8>>) -> Result<()> {
    let data = Invmsg {
        addr_from: server.node_address.clone(),
        kind: kind.to_string(),
        items,
    };
    let data = serialize(&(cmd_to_bytes("inv"), data))?;
    send_data(server, addr, &data)
}

fn send_get_blocks(server: Server, addr: &str) -> Result<()> {
    let data = GetBlocksmsg {
        addr_from: addr.to_string(),
    };
    let data = serialize(&(cmd_to_bytes("getblocks"), data))?;
    send_data(server, addr, &data)
}

fn send_get_data(server: Server, addr: &str, kind: &str, id: &str) -> Result<()> {
    let data = GetDatamsg {
        addr_from: server.node_address.clone(),
        kind: kind.to_string(),
        id: id.to_string(),
    };
    let data = serialize(&(cmd_to_bytes("getdata"), data))?;
    send_data(server, addr, &data)
}

fn send_tx(server: Server, addr: &str, tx: &Transaction) -> Result<()> {
    let data = Txmsg {
        addr_from: server.node_address.clone(),
        transaction: tx.clone(),
    };
    let data = serialize(&(cmd_to_bytes("tx"), data))?;
    send_data(server, addr, &data)
}

fn send_version(server: Server, addr: &str) -> Result<()> {
    let data = Versionmsg {
        addr_from: server.node_address.clone(),
        best_height: server.get_best_height()?,
        version: VERSION,
    };
    let data = serialize(&(cmd_to_bytes("version"), data))?;
    send_data(server, addr, &data)
}

fn handle_version(server: Server, msg: Versionmsg) -> Result<()> {
    Ok(())
}

fn handle_addr(server: Server, msg: Vec<String>) -> Result<()> {
    Ok(())
}

fn handle_block(server: Server, msg: Blockmsg) -> Result<()> {
    Ok(())
}

fn handle_inv(server: Server, msg: Invmsg) -> Result<()> {
    Ok(())
}

fn handle_get_blocks(server: Server, msg: GetBlocksmsg) -> Result<()> {
    Ok(())
}

fn handle_get_data(server: Server, msg: GetDatamsg) -> Result<()> {
    Ok(())
}

fn handle_tx(server: Server, msg: Txmsg) -> Result<()> {
    Ok(())
}

fn handle_connection(mut stream: TcpStream, server: Server) -> Result<()> {
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

fn cmd_to_bytes(cmd: &str) -> [u8; CMD_LEN] {
    let mut data = [0; CMD_LEN];
    for (i, d) in cmd.as_bytes().iter().enumerate() {
        data[i] = *d;
    }
    data
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
