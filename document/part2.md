# rust 从零开始构建区块链(Bitcoin)系列 - 交易和一些辅助工具

Github链接，包含文档和全部代码：

[https://github.com/yunwei37/blockchain-rust](https://github.com/yunwei37/blockchain-rust)

这篇文章对应着 Go 原文的第三部分和第四部分，包括 `持久化和命令行接口` 以及 `交易`。交易是一个由区块链构成的虚拟货币系统的核心，但在讨论交易之前，我们还会先着手做一些辅助的工具部分：

- 将区块链持久化到一个数据库中（在内存中肯定是不现实的）；
- 提供一点命令行交互的接口；
- 除了上述在 Go 原文中的两个方面，我们还会添加一个 Rust 的日志功能，这部分在现在看来用处可能不大，但在网络交互中还是非常有必要的。

## 一些辅助工具

本阶段的代码实现在这里：[commit e2094c0](https://github.com/yunwei37/blockchain-rust/tree/e2094c0ef94fadc4e01030312a1ad890ec633d6f)

### 数据库

首先来看数据库部分，这里我选择了一个简单的 Rust 键值对数据库 [sled](https://github.com/spacejam/sled):

>Sled是基于Bw树构建的嵌入式KV数据库，其API接近于一个线程安全的BTreeMap<[u8], [u8]>。而其Bw树的数据结构加上包括crossbeam-epoch的“GC”等技术，使得Sled成为一个lock-free的数据库而在并发环境中傲视群雄。官方宣称在一台16核的机器上，在一个小数据集上可以达到每分钟10亿次操作（95%读核5%写）。

示例代码可以参考上面那个链接里面的 README 文件，它的 API 也比原文 Go 里面的更简单。这里，我们使用的键值对有两种类型：

- 32 字节的 block-hash -> block 结构体
- 'LAST' -> 链中最后一个块的 hash

实际存储的全部都是序列化的数据，这里我们可以使用 [serde](https://github.com/serde-rs/serde) 进行序列化和反序列化的操作：

>serde, 是rust语言用来序列化和反序列化数据的一个非常高效的解决方案。
>本质上，serde提供了一个序列化层的概念。可以将任何支持的数据格式反序列化成中间格式，然后序列化成任何一种支持的数据格式。

来看看代码吧！
 
首先，我们修改一下 Blockchain 数据结构的定义，在其中保存有一个 sled 数据库的实例：

```rs
#[derive(Debug)]
pub struct Blockchain {
    tip: String,
    db: sled::Db,
}
```

由于我们需要对 Block 结构体进行序列化，我们可以很简单地使用 serde 库的 derive 特性机制，只要在结构体上面添加上 `#[derive(Serialize, Deserialize)` 就好：

```rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    timestamp: u128,
    data: String,
    prev_block_hash: String,
    hash: String,
    nonce: i32,
}
```

然后，我们可以这样创建一个 Blockchain 结构体，如果存在数据库（找到了 `LAST` 键对应的内容）的话就加载对应的区块链，不存在就创建之：

```rs
impl Blockchain {
    pub fn new() -> Result<Blockchain> {
        info!("Creating new blockchain");

        let db = sled::open("data/blocks")?;
        match db.get("LAST")? {
            Some(hash) => {                         // 如果存在数据库（找到了 `LAST` 键对应的内容）
                info!("Found block database");
                let lasthash = String::from_utf8(hash.to_vec())?;
                Ok(Blockchain {
                    tip: lasthash.clone(),
                    current_hash: lasthash,
                    db,
                })
            }
            None => {                               // 数据库里面没有 LAST ，创建一个新的区块链
                info!("Creating new block database");
                let block = Block::new_genesis_block();
                db.insert(block.get_hash(), serialize(&block)?)?;       // 序列化然后插入
                db.insert("LAST", block.get_hash().as_bytes())?;
                let bc = Blockchain {
                    tip: block.get_hash(),
                    db,
                };
                bc.db.flush()?;
                Ok(bc)
            }
        }
    }   
```

然后是向数据库里面添加区块，代码的逻辑很简单：找到 LAST 对应的区块，把它作为上个区块来创建新的区块，然后插入数据库，更新 LAST：

```rs
impl Blockchain {
    pub fn add_block(&mut self, data: String) -> Result<()> {
        info!("add new block to the chain");

        let lasthash = self.db.get("LAST")?.unwrap();

        let newblock = Block::new_block(data, String::from_utf8(lasthash.to_vec())?)?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();

        Ok(())
    }
```

为了让调用比较方便，我们可以考虑为区块链增加一个迭代器，这个迭代器能够顺序打印区块链（在数据库中是按hash存储的）。具体代码这里就忽略过去啦（毕竟也不算特别重要，可以参考源代码 blockchain.rs）。

### 命令行接口和日志

rust 没有 Go 那样原生强大的标准库，但我们还是有很多社区库可以用的，比如这个 [clap](https://github.com/clap-rs/clap)：

> clap是一个简单易用，功能强大的命令行参数解析库，用于解析并验证用户在运行命令行程序时提供的命令行参数字符串。 你所需要做的只是提供有效参数的列表，clap会自动处理其余的繁杂工作。 这样工程师可以把时间和精力放在实现程序功能上，而不是参数的解析和验证上。

我们这里创建了一个简单的结构体 `Cli`：

```rs
pub struct Cli {
    bc: Blockchain,
}
```

大概 clap 匹配命令的代码是这样的：

```rs
impl Cli {
    pub fn run(&mut self) -> Result<()> {
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
                self.bc.add_block(data)?;
            } else {
                println!("Not data...");
            }
        }

        if let Some(_) = matches.subcommand_matches("printchain") {
            let bc = Blockchain::new()?;
            for b in bc.iter() {
                println!("block: {:#?}", b);
            }
        }

        ....
```

我们在这里创建了两个子命令:

- `addblock <data>` 添加一个新的区块
- `printchain` 使用迭代打印区块链

最后，我们可以使用 [env_logger](http://www.eclipse.org/paho/files/rustdoc/env_logger/index.html) 来进行日志记录，具体实例就像上面提到的那样: `info!("add new block to the chain");`

在 main 函数中初始化 env_logger，指定默认日志等级为warnning，然后运行命令行处理程序：

```rs
fn main() -> Result<()> {
    env_logger::from_env(Env::default().default_filter_or("warning")).init();

    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}
```

这样我们就有一个看起来还行的 blockchain demo 啦，下一步就是加上交易！

## 交易

交易（transaction）是比特币的核心所在，而比特币使用区块链唯一的目的，也正是为了能够安全可靠地存储交易。在区块链中，交易一旦被创建，就没有任何人能够再去修改或是删除它；同时，由于比特币采用的是 UTXO 模型，并非账户模型，并不直接存在“余额”这个概念，余额需要通过遍历整个交易历史得来。

详细的信息可以参考：[en.bitcoin.it/wiki/Transaction](https://en.bitcoin.it/wiki/Transaction)

也可以对照原版的中文翻译看，他那关于原理的介绍比较详细：[transactions-1](https://liuchengxu.gitbook.io/blockchain/bitcoin/transactions-1#yin-yan)

首先，我们看看交易的数据结构，一笔交易由一些输入（input）和输出（output）组合而来：

```rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}
```

对于每一笔新的交易，它的输入会引用（reference）之前一笔交易的输出，不过：

- 有一些输出并没有被关联到某个输入上，对应着 coinbase；
- 一笔交易的输入可以引用之前多笔交易的输出；
- 一个输入必须引用一个输出；

这是输出，包含一定量的比特币和一个锁定脚本（这里并不会实现全面）：

```rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub script_pub_key: String,
}
```

这是输入，引用（reference）之前一笔交易的输出：

```rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub script_sig: String,
}

```