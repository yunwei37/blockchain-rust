#! https://zhuanlan.zhihu.com/p/256444986
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

这是输出，包含一定量的比特币和一个锁定脚本（这里并不会实现全面的脚本语言）：

```rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub script_pub_key: String,
}
```

我们将只会简单地把输出和用户定义的钱包地址（一个任意的字符串）作比较：

```rs
impl TXOutput {
    /// CanBeUnlockedWith checks if the output can be unlocked with the provided data
    pub fn can_be_unlock_with(&self, unlockingData: &str) -> bool {
        self.script_pub_key == unlockingData
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

事实上，虚拟货币就是存储在输出中里面。由于还没有实现地址（address），所以目前我们会避免涉及逻辑相关的完整脚本。

来看看一个基本的使用场景：从一个用户向另外一个用户发送币，即创建一笔交易：

之前我们仅仅实现了简单的 `coinbase` 交易方法，也就是挖矿，现在我们需要一种通用的普通交易：

- 首先，我们需要使用 `find_spendable_outputs` 函数，找到发送方可以花费的货币数量，以及包含这些货币的未使用输出；
- 然后，我们使用这些输出创建一个新的输入给接收方，这里已经被引用的输入就相当于被花掉了；注意，输出是不可再分的；
- 最后，将多余的钱（找零）创建一个新的输出返回给发送方。

好啦！一个最基本的交易原型就这样完成了，现在来看看代码；

```rs
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
```

来看看相关的辅助函数，首先是在区块链中寻找未花费的输出 find_spendable_outputs，该方法返回一个包含累积未花费输出和相关输出结构体集合的元组：

- 首先，使用 `find_unspent_transactions` 找到包含发送方的所有未花掉的输出；
- 然后，在所有未花掉的输出上面迭代，将能够被使用者解锁的输出的 id 插入到用交易 id 作为索引的集合中；增加累计数值；
- 当累计数值超过需要数值的时候返回。

```rs
impl Blockchain {
    pub fn find_spendable_outputs(
        &self,
        address: &str,
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;
        let unspend_TXs = self.find_unspent_transactions(address);

        for tx in unspend_TXs {
            for index in 0..tx.vout.len() {
                if tx.vout[index].can_be_unlock_with(address) && accumulated < amount {
                    match unspent_outputs.get_mut(&tx.id) {
                        Some(v) => v.push(index as i32),
                        None => {
                            unspent_outputs.insert(tx.id.clone(), vec![index as i32]);
                        }
                    }
                    accumulated += tx.vout[index].value;

                    if accumulated >= amount {
                        return (accumulated, unspent_outputs);
                    }
                }
            }
        }
        (accumulated, unspent_outputs)
    }
```

下一步是找到区块链中对应地址能解锁的包含未花费输出的交易，即 `find_unspent_transactions`：

- 如果一个输出可以被发送方的地址解锁，并且该输出没有被包含在一个交易的输入中，它就是可以使用的；
- 由于我们对区块链是从尾部往头部迭代，因此如果我们见到的输出没有被包含在我们见到的任何一笔输入中，它就是未使用的；
- 我们将见到的输入加入一个集合，然后在这个集合中查找对应的输出，如果一个输出可以被解锁并且没有在集合中找到的话，它就是可以被花费的。

```rs
impl Blockchain {
    fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
        let mut spent_TXOs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut unspend_TXs: Vec<Transaction> = Vec::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spent_TXOs.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    if tx.vout[index].can_be_unlock_with(address) {
                        unspend_TXs.push(tx.to_owned())
                    }
                }

                if !tx.is_coinbase() {
                    for i in &tx.vin {
                        if i.can_unlock_output_with(address) {
                            match spent_TXOs.get_mut(&i.txid) {
                                Some(v) => {
                                    v.push(i.vout);
                                }
                                None => {
                                    spent_TXOs.insert(i.txid.clone(), vec![i.vout]);
                                }
                            }
                        }
                    }
                }
            }
        }

        unspend_TXs
    }

```

对于一个普通的交易，可以用以上方法完成；但我们还需要一种 `coinbase` 交易，它“凭空”产生了币，这是矿工获得挖出新块的奖励；

```rs
impl Transaction {
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
```

我们还可以创建一个简单的辅助函数，让我们可以比较简单地获取余额：这个函数返回了一个交易列表，里面包含了未花费输出；

```rs
impl Blockchain {
    /// FindUTXO finds and returns all unspent transaction outputs
    pub fn find_UTXO(&self, address: &str) -> Vec<TXOutput> {
        let mut utxos = Vec::<TXOutput>::new();
        let unspend_TXs = self.find_unspent_transactions(address);
        for tx in unspend_TXs {
            for out in &tx.vout {
                if out.can_be_unlock_with(&address) {
                    utxos.push(out.clone());
                }
            }
        }
        utxos
    }
```

交易的部分差不多就这些啦！我们已经完成了准备工作，现在可以更改一下之前留下来的接口：

我们首先需要在区块中添加一下包含的交易，是这样的：

```rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    nonce: i32,
}

```

然后也要更改一下添加区块的接口：

```rs

impl Block {
    pub fn new_block(transactions: Vec<Transaction>, prev_block_hash: String) -> Result<Block> {
        ....
    }

    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new_block(vec![coinbase], String::new()).unwrap()
    }
```

在创建区块链的时候，我们也需要创建一笔 `coinbase` 交易：

```rs
impl Blockchain {
    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        ...
        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);
        ...
    }
```

基本上大功告成！我们看看具体的命令实现：

send 命令：

```rs
        ...
            let mut bc = Blockchain::new()?;
            let tx = Transaction::new_UTXO(from, to, amount, &bc)?;
            bc.mine_block(vec![tx])?;
        ...
```

getbalance 命令：

```rs
        ...
            let bc = Blockchain::new()?;
            let utxos = bc.find_UTXO(&address);

            let mut balance = 0;
            for out in utxos {
                balance += out.value;
            }
            println!("Balance of '{}': {}\n", address, balance);
        ...
```

这样就好啦！如果想要进一步观察交易的相关知识，可以参考：[https://en.bitcoin.it/wiki/Transaction](https://en.bitcoin.it/wiki/Transaction)

