#! https://zhuanlan.zhihu.com/p/237854049
# 用 rust 从零开始构建区块链(Bitcoin)系列 - 基本原型和工作量证明算法

Github链接，包含文档和全部代码：[https://github.com/yunwei37/blockchain-rust](https://github.com/yunwei37/blockchain-rust)

区块链是 21 世纪最具革命性的技术之一，尽管区块链的热潮已经褪去，但不可否认的是它确实有其存在的意义和价值：区块链的本质是一个分布式记账和存储系统，一种无法被篡改的数据结构，它也应当会成为未来在金融和政治方面的某种信息基础设施之一。当然，肯定和各种空气币无关；目前我们所提出来的大多数应用也只是一个比较的设想而已，它的应用并没有那么广泛。

作为一个强调安全性的语言，Rust 和区块链的应用方向是十分契合的，也可以看到越来越多的区块链项目使用 Rust 进行开发。本文将使用 Rust 构建一个简单的区块链模型，并基于它来构建一个简化版的加密货币（参考比特币的实现），以供学习之用。大部分设计参考自Github上面使用Go语言完成的另外一个项目：blockchain_go，和 go 相比，Rust 的开发效率并不会低太多，但安全性和运行速度还是要好不少的。另外，得益于 Rust 在区块链相关部分的社区看起来还是挺繁荣的（？），我们很多简单的组件也可以采用开源库代替，例如地址生成等等。

>本文不会涉及太多的区块链基础知识，具体可以参考原文的中文翻译：[liuchengxu.gitbook.io/blockchain/](https://liuchengxu.gitbook.io/blockchain/)
>阅读本文需要您对 rust 事先有一定了解

第一篇文章希望讨论区块链最基本的数据结构原型，以及挖矿的本质 ---- 工作量证明算法

## 基本原型

本阶段的代码实现在这里：[github.com/yunwei37/blockchain-rust/tree/bd0efe7f4105a3daafd9311d3dd643482b63cb84](https://github.com/yunwei37/blockchain-rust/tree/bd0efe7f4105a3daafd9311d3dd643482b63cb84)

区块链本质是一个很简单的概念。

在区块链中，真正存储有效信息的是区块（block），这里是个简单的定义: 

```rs
/// Block keeps block headers
#[derive(Debug)]
pub struct Block {
    timestamp: u128,
    data: String,
    prev_block_hash: String,
    hash: String,
}
```

通过在每个区块中间保存上一个区块的哈希值，就可以保证链条中间的某一个区块不会被篡改；

参数：

- Timestamp 当前时间戳，也就是区块创建的时间
- data 区块存储的实际有效信息，也就是交易
- prev_block_hash 前一个块的哈希，即父哈希
- hash 当前块的哈希

可以通过调用 `set_hash` 方法来计算区块哈希值，这里采用 SHA-256 算法进行：取 Block 结构的部分字段（Timestamp, Data 和 PrevBlockHash），并将它们相互拼接起来，然后在拼接后的结果上计算一个 SHA-256 值，并赋值给 hash 字段：

`block.rs`:

```rs
impl Block {
    /// SetHash calculates and sets block hash
    pub fn set_hash(&mut self) -> Result<()> {
        self.timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let content = (self.data.clone(),self.prev_block_hash.clone(), self.timestamp);
        let bytes = serialize(&content)?;
        let mut hasher = Sha256::new();
        hasher.input(&bytes[..]);
        self.hash = hasher.result_str();
        Ok(())
    }
}
```

我们需要一个方法来封装创建一个块：

`block.rs`:

```rs
impl Block {
    /// NewBlock creates and returns Block
    pub fn new_block(data: String, prev_block_hash: String) -> Result<Block> {
        let mut block = Block {
            timestamp: 0,
            data,
            prev_block_hash,
            hash: String::new(),
        };
        block.set_hash()?;
        Ok(block)
    }
}
```

有了区块，接下来可以看一下区块链：本质上来说：区块链就是一个有着特定结构的数据库，是一个有序，每一个块都连接到前一个块的链表，只是采用哈希值取代了指针进行链接。

这里可以采用 vec 进行存储

`blockchain.rs`:

```rs
/// Blockchain keeps a sequence of Blocks
#[derive(Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
}
```

接下来看看如何添加一个区块：

`blockchain.rs`:

```rs
impl Blockchain {
    /// AddBlock saves provided data as a block in the blockchain
    pub fn add_block(&mut self, data: String) -> Result<()> {
        let prev = self.blocks.last().unwrap();
        let newblock = Block::new_block(data, prev.get_hash())?;
        self.blocks.push(newblock);
        Ok(())
    }
}
```

好啦！基本工作就这些！不过还有一点，为了加入一个新的块，我们需要有一个已有的块，在初始情况下就需要一个创世块：

`block.rs`:

```rs
impl Block {
    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block() -> Block {
        Block::new_block(String::from("Genesis Block"), String::new()).unwrap()
    }
}
```

创建一个区块链的函数接口：

`blockchain.rs`:

```rs
impl Blockchain {
    /// NewBlockchain creates a new Blockchain with genesis Block
    pub fn new() -> Blockchain {
        Blockchain {
            blocks: vec![Block::new_genesis_block()],
        }
    }
}
```

结束！

## 工作量证明：

上面那个是单机版的，从分布式系统的角度上来讲，每个主机都可能自行往区块链中添加区块，如何协调一致性和保证系统不会被攻击就是一个大问题。

比特币采用的是 PoW 算法，要让一个人必须经过一系列困难的工作，才能将数据放入到区块链中，完成这个工作的人，也会获得相应奖励（这也就是通过挖矿获得币）。并不只有这个算法有效，具体关于分布式的内容，可以参考后面的网络部分。

这里具体的算法是 Hashcash，可以参考维基：[https://en.wikipedia.org/wiki/Hashcash](https://en.wikipedia.org/wiki/Hashcash)

步骤：

1. 取一些公开的数据（比如，如果是 email 的话，它可以是接收者的邮件地址；在比特币中，它是区块头）
2. 给这个公开数据添加一个计数器。计数器默认从 0 开始
3. 将 data(数据) 和 counter(计数器) 组合到一起，获得一个哈希
4. 检查哈希是否符合一定的条件：
    - 如果符合条件，结束
    - 如果不符合，增加计数器，重复步骤 3-4

我们继续在 `block.rs` 中写代码：

首先，在 block 里面增加一个计数器：

```rs
/// Block keeps block headers
pub struct Block {
    ...
    nonce: i32,
}
```

写个辅助函数，获取需要被哈希的数据序列值：

```rs
impl Block {
    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.data.clone(),
            self.timestamp,
            TARGET_HEXS,
            self.nonce,
        );
        let bytes = serialize(&content)?;
        Ok(bytes)
    }
}
```

然后，判断当前的哈希值是否满足要求：

```rs
const TARGET_HEXS: usize = 4;

impl Block {
    /// Validate validates block's PoW
    fn validate(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        let mut vec1: Vec<u8> = Vec::new();
        vec1.resize(TARGET_HEXS, '0' as u8);
        Ok(&hasher.result_str()[0..TARGET_HEXS] == String::from_utf8(vec1)?)
    }
}
```

然后，就可以跑算法啦：

```rs
impl Block {
    /// Run performs a proof-of-work
    fn run_proof_of_work(&mut self) -> Result<()> {
        println!("Mining the block containing \"{}\"\n", self.data);
        while !self.validate()? {
            self.nonce += 1;
        }
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }
}
```

这样我们就完成了工作量证明也就是挖矿的程序啦！

在 main 里面写个测试程序，我们可以用 Debug 宏打印区块链：

```rs
fn main() -> Result<()> {
    let mut bc = Blockchain::new();
    sleep(Duration::from_millis(10));
    bc.add_block(String::from("Send 1 BTC to Ivan"))?;
    sleep(Duration::from_millis(30));
    bc.add_block(String::from("Send 2 more BTC to Ivan"))?;

    println!("Blockchain: {:#?}", bc);
    Ok(())
}
```

输出：

```json
Mining the block containing "Genesis Block"

Mining the block containing "Send 1 BTC to Ivan"

Mining the block containing "Send 2 more BTC to Ivan"

Blockchain: Blockchain {
    blocks: [
        Block {
            timestamp: 1599905545625,
            data: "Genesis Block",
            prev_block_hash: "",
            hash: "0000f81cad3bda84526e742a2931bd94ac689c3795ee2da713f8e3bf5d6b461a",
            nonce: 47246,
        },
        Block {
            timestamp: 1599905546544,
            data: "Send 1 BTC to Ivan",
            prev_block_hash: "0000f81cad3bda84526e742a2931bd94ac689c3795ee2da713f8e3bf5d6b461a",
            hash: "00008e9348e0500ff0324bbc0b861f5a01033ac317a12b28987675b5906bf03e",
            nonce: 31604,
        },
        Block {
            timestamp: 1599905547428,
            data: "Send 2 more BTC to Ivan",
            prev_block_hash: "00008e9348e0500ff0324bbc0b861f5a01033ac317a12b28987675b5906bf03e",
            hash: "0000c6d5e4f800116be7551ba0afe01174eebadc6897edc9dc2090b6fb387096",
            nonce: 24834,
        },
    ],
}

```

## 参考资料：

- 源代码：[github.com/yunwei37/blockchain-rust](https://github.com/yunwei37/blockchain-rust)
- Go 原版代码：[https://github.com/Jeiwan/blockchain_go/tree/part_2](https://github.com/Jeiwan/blockchain_go/tree/part_2)
- 区块链理论学习入门指南：[daimajia.com/2017/08/24/how-to-start-blockchain-learning](https://daimajia.com/2017/08/24/how-to-start-blockchain-learning)
- <<区块链技术指南>>: [yeasy.gitbooks.io/blockchain_guide/content](https://yeasy.gitbooks.io/blockchain_guide/content)