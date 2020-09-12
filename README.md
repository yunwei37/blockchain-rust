# blockchain-rust - 用 rust 从零开始构建区块链(Bitcoin)系列

[![Actions Status](https://github.com/yunwei37/blockchain-demo/workflows/CI/badge.svg)](https://github.com/yunwei37/blockchain-demo/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

reimplement `blockchain_go` in rust, and not only blockchain_go;

a simple blockchain demo for learning

## the code for each article

1. part1: Basic Prototype     `基本原型`              [commit bd0efe7](https://github.com/yunwei37/blockchain-rust/tree/bd0efe7f4105a3daafd9311d3dd643482b63cb84)
2. part2: Proof-of-Work       `工作量证明`            [commit 9d9370a](https://github.com/yunwei37/blockchain-rust/tree/9d9370aa22af34244659034918f2aad4a2cb96d2)
3. part3: Persistence and CLI` 持久化、命令行、日志`  [commit e2094c0](https://github.com/yunwei37/blockchain-rust/tree/e2094c0ef94fadc4e01030312a1ad890ec633d6f)
4. part4: Transactions 1      `交易（1）`  [commit bdbdcec](https://github.com/yunwei37/blockchain-rust/tree/bdbdcec8b79d5ff701d207f67a1b68849a35d865)
5. part5: Addresses           `地址和签名`  [commit 440cba2](https://github.com/yunwei37/blockchain-rust/tree/440cba230cbd81957c3285b21c705a5708ed2b5b)
6. part6: Transactions 2      `交易（2）`  [commit 4912743](https://github.com/yunwei37/blockchain-rust/tree/4912743daa2699cb8c0c4ba5bf136534b272cecd)
7. part7: Network      `网络和分布式一致性算法` master

## Chinese Documents

- 基本原型和工作量证明算法: [part1.md](document/part1.md)

## usage

- Create wallet: 
  ```sh
  cargo run createwallet
  ```
- Create blockchain:
  ```
  cargo run createblockchain <address>
  ```
- send coins (if `-m` is specified, the block will be mined immediately in the same node):
  ```
  cargo run send <from> <to> <amount> -m 
  ```
- start server:
  ```
  cargo run startnode <port>
  ```
  or start miner node:
  ```
  cargo run startminer <port> <address>
  ```
- get balance:
  ```
  cargo run getbalance <address>
  ```

You can use the `RUST_LOG=info` to print the log.

## reference

- `blockchain_go` code: [https://github.com/Jeiwan/blockchain_go](https://github.com/Jeiwan/blockchain_go)
- Build a cryptocurrency! - Blockchain in Rust: [https://github.com/GeekLaunch/blockchain-rust](https://github.com/GeekLaunch/blockchain-rust)
- 中文版文档：[https://liuchengxu.gitbook.io/blockchain/](https://liuchengxu.gitbook.io/blockchain/)