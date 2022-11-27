/*  
    block_chain.rs.rs
    Copyright (c) 2022 Yasuo Tabei
 
    Released under the GNU General Public License version 3.0
    see https://www.gnu.org/licenses/gpl-3.0.en.html

    The GNU General Public License is a free, copyleft license for software and other kinds of works.
    The licenses for most software and other practical works are designed to take away your freedom to share and change the works. By contrast, the GNU General Public License is intended to guarantee your freedom to share and change all versions of a program--to make sure it remains free software for all its users. We, the Free Software Foundation, use the GNU General Public License for most of our software; it applies also to any other work released this way by its authors. You can apply it to your programs, too.
    When we speak of free software, we are referring to freedom, not price. Our General Public Licenses are designed to make sure that you have the freedom to distribute copies of free software (and charge for them if you wish), that you receive source code or can get it if you want it, that you can change the software or use pieces of it in new free programs, and that you know you can do these things.
    To protect your rights, we need to prevent others from denying you these rights or asking you to surrender the rights. Therefore, you have certain responsibilities if you distribute copies of the software, or if you modify it: responsibilities to respect the freedom of others.
    For example, if you distribute copies of such a program, whether gratis or for a fee, you must pass on to the recipients the same freedoms that you received. You must make sure that they, too, receive or can get the source code. And you must show them these terms so they know their rights.
    Developers that use the GNU GPL protect your rights with two steps: (1) assert copyright on the software, and (2) offer you this License giving you legal permission to copy, distribute and/or modify it.
    For the developers' and authors' protection, the GPL clearly explains that there is no warranty for this free software. For both users' and authors' sake, the GPL requires that modified versions be marked as changed, so that their problems will not be attributed erroneously to authors of previous versions.
    Some devices are designed to deny users access to install or run modified versions of the software inside them, although the manufacturer can do so. This is fundamentally incompatible with the aim of protecting users' freedom to change the software. The systematic pattern of such abuse occurs in the area of products for individuals to use, which is precisely where it is most unacceptable. Therefore, we have designed this version of the GPL to prohibit the practice for those products. If such problems arise substantially in other domains, we stand ready to extend this provision to those domains in future versions of the GPL, as needed to protect the freedom of users.
    Finally, every program is threatened constantly by software patents. States should not allow patents to restrict development and use of software on general-purpose computers, but in those that do, we wish to avoid the special danger that patents applied to a free program could make it effectively proprietary. To prevent this, the GPL assures that patents cannot be used to render the program non-free.
    The precise terms and conditions for copying, distribution and modification follow.
*/

//! The file includes four structs of TransactionOutput, TransactionInput and Transaction

use chrono::prelude::*;

extern crate crypto;
use crypto::ed25519::keypair;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

//use std::collections::linked_list::CursorMut;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

extern crate serde;
extern crate serde_json;
use serde::{Deserialize, Serialize, Serializer};


pub mod keymanager;

pub const DIFFICULTY: usize = 3;

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionOutput {
    pub recipient: String, 
    pub value: i64
}

impl TransactionOutput {
    pub fn create(recipient_address: &str, value: i64) -> TransactionOutput {
        TransactionOutput {
            recipient: recipient_address.to_string(),
            value: value
        }
    }

    pub fn clone(&self) -> TransactionOutput {
        TransactionOutput {
            recipient: self.recipient.clone(), 
            value: self.value
        }
    }

    pub fn to_string(&self) -> String{
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(msg: &str) -> TransactionOutput {
        serde_json::from_str(&msg).unwrap()
    }

    pub fn equal(&self, tx_out: &TransactionOutput) -> bool {
        if self.recipient.eq(&tx_out.recipient) && self.value == tx_out.value {
            return true;
        }
        return false;
    }

    pub fn print(&self) {
        println!("transactionOutput");
        println!("{}", self.recipient);
        println!("{}", self.value);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionInput {
    pub transaction: Box<Transaction>, 
    pub output_index: usize
}

impl TransactionInput {
    pub fn create(transaction: Transaction, output_index: usize) -> TransactionInput {
        TransactionInput {
            transaction: Box::new(transaction),
            output_index: output_index
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_str(msg: &str) -> TransactionInput {
        serde_json::from_str(msg).unwrap()
    }

    pub fn clone(&self) -> TransactionInput {
        TransactionInput {
            transaction: Box::new(self.transaction.clone()), 
            output_index: self.output_index
        }
    }

    pub fn equal(&self, tx_in: &TransactionInput) -> bool {
        if self.to_string().eq(&tx_in.to_string()) == true {
            return true;
        }
        return false;
    }

    pub fn print(&self) {
        println!("TransactionInput");
//        println!("transaction: {:?}", self.transaction);
        println!("output_index: {}", self.output_index);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub timestamp: String,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>, 
    pub signature: String, 
    pub tx_type: bool
}

impl Transaction {
    pub fn create() -> Transaction {
        Transaction {
            timestamp: Utc::now().to_string(),
            inputs: Vec::new(), 
            outputs: Vec::new(),
            signature: String::new(), 
            tx_type: false
        }
    }

    pub fn create_for_genesis_block() -> Transaction {
        Transaction {
            timestamp: "0".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(), 
            signature: String::new(), 
            tx_type: false
        }
    }

    pub fn clone(&self) -> Transaction {
        let mut inputs_clone: Vec<TransactionInput> = Vec::new();
        for input in self.inputs.iter() {
            inputs_clone.push(input.clone());
        }

        let mut outputs_clone: Vec<TransactionOutput> = Vec::new();
        for output in self.outputs.iter() {
            outputs_clone.push(output.clone());
        }

        Transaction {
            timestamp: self.timestamp.clone(),
            inputs: inputs_clone, 
            outputs: outputs_clone, 
            signature: self.signature.clone(), 
            tx_type: self.tx_type
        }
    }

    pub fn create_coinbase_transaction(recipient_address: &str, value: i64) -> Transaction {
        let output   = TransactionOutput::create(recipient_address, value);
        let mut outputs: Vec<TransactionOutput> = Vec::new();
        outputs.push(output);

        Transaction {
            timestamp: Utc::now().to_string(),
            inputs: Vec::new(), 
            outputs: outputs, 
            signature: "".to_string(), 
            tx_type: true
        }
    }

    pub fn is_enough_inputs(&self, fee: i64) -> bool {
        let mut total_in: i64 = 0;

        for input_iter in self.inputs.iter() {
            total_in += self.outputs[input_iter.output_index].value;
        }

        let mut total_out: i64 = 0;
        for output in self.outputs.iter() {
            total_out += output.value;
        }

        println!("total_in: {}", total_in);
        println!("total_out: {}", total_out);

        let delta = total_in - total_out - fee;
        if delta >= 0 {
            return true;
        }
        return false;
    }

    pub fn compute_change(&self, fee: i64) -> i64 {
        let mut total_in: i64 = 0;
        for input in self.inputs.iter() {
            total_in += self.outputs[input.output_index].value;
        }

        let mut total_out: i64 = 0;
        for output in self.outputs.iter() {
            total_out += output.value;
        }

        let delta: i64 = total_in - total_out - fee;
        return delta;
    }

    pub fn equal(&self, transaction: &Transaction) -> bool {
        if self.to_string() == transaction.to_string() {
            return true;
        }
        return false;
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_str(msg: &str) -> Transaction {
        serde_json::from_str(msg).unwrap()
    }

    pub fn print(&self) {
        println!("{}", self.to_string());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionPool {
    pub transactions: Vec<Transaction>
}

impl TransactionPool {
    pub fn create() -> TransactionPool {
        println!("create transaction pool");
        TransactionPool {
            transactions: Vec::new()
        }        
    }

    pub fn push(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn clone(&self) -> TransactionPool {
        let mut transactions_clone:Vec<Transaction> = Vec::new();
        for t in &self.transactions {
            transactions_clone.push(t.clone());
        }
        TransactionPool {
            transactions: transactions_clone
        }
    }

    pub fn clear(&mut self) {
        self.transactions.clear();
    }

    pub fn len(&self) -> usize {
        return self.transactions.len();
    }

    pub fn set_new_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn contain(&self, transaction: &Transaction) -> bool {
        for t in &self.transactions {
            if t.equal(transaction) == true {
                return true;
            }
        }
        return false;

    }
/*
    pub fn has_this_output_in_my_tp(&self, transaction_output: TransactionOutput) -> bool {
        println!("has_this_output_in_my_tp is called!");

        let transactions = &self.transactions;

        for t in transactions.iter() {
            let input_t = &t.inputs;
            for it in input_t.iter() {
                if it.equal(&transaction_output) == true {
                    return true;
                }
            }
        }
        return false;
    }
    */
/*
    pub fn clear_my_transactions(&self, index: usize) -> TransactionPool {
        let mut newtransactions: Vec<Transaction> = Vec::new();
        for i in index..self.transactions.len() {
            let tmp = Transaction::create(&self.transactions[i].sender, &self.transactions[i].recipient, self.transactions[i].value);
            newtransactions.push(tmp);
        }
        TransactionPool {
            transactions: newtransactions
        }
    }
*/
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(msg: &str) -> TransactionPool {
        serde_json::from_str(msg).unwrap()
    }

    pub fn equal(&self, tp: &TransactionPool) -> bool {
        if self.transactions.len() != tp.transactions.len() {
            return false;
        }
        for i in 0..self.transactions.len() {
            if self.transactions[i].equal(&tp.transactions[i]) == false {
                return false;
            }
        }
        return true;
    }

    pub fn print(&self) {
        for transaction in &self.transactions {
            transaction.print();
        }
    }

    pub fn get_total_fee_from_tp(&self) -> i64 {
        println!("get_total_fee_from_tp is called!");

        let transactions = &self.transactions;
        let mut result:i64 = 0;

        for t in transactions.iter() {
            let mut total_in: i64 = 0;
            for i in t.inputs.iter() {
                total_in += i.transaction.outputs[i.output_index].value;
            }
            let mut total_out: i64 = 0;
            for o in t.outputs.iter() {
                total_out += o.value;
            }
            let delta:i64 = total_in - total_out;
            result += delta;
        }
        return result;

    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
//    pub timestamp: DateTime<Utc>,
    pub timestamp: String,
    pub transaction_pool: TransactionPool,
    pub previous_block: String,
    pub nonce: u64
}

impl Block {
    pub fn create(transaction_pool: TransactionPool, previous_block: String) -> Block {
        let mut block = Block {
            timestamp: Utc::now().to_string(), 
            transaction_pool: transaction_pool, 
            previous_block: previous_block,
            nonce: 0
        };
        block
    }

    pub fn clone(&self) -> Block {
        Block {
            timestamp: self.timestamp.clone(), 
            transaction_pool: self.transaction_pool.clone(),
            previous_block: self.previous_block.clone(), 
            nonce: self.nonce
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(msg: &str) -> Block {
        serde_json::from_str(msg).unwrap()
    }

    pub fn get_hash(&self) -> String {
        get_double_sha256(&self.to_string())
    }

    pub fn create_genesis_block() -> Block {
        let transaction = Transaction::create_for_genesis_block();
        let mut transaction_pool = TransactionPool::create();
        transaction_pool.transactions.push(transaction);
        let block = Block {
            timestamp: "0".to_string(),
            transaction_pool: transaction_pool, 
            previous_block: "".to_string(),
            nonce: 0
        };
        block
    }

    pub fn compute_nonce_for_pow(&mut self, to_stop: &Arc<AtomicBool>) -> bool {
        println!("start computing nonce");
        let mut nonce: u64 = 0;
        let suffix: String = "0".to_string().repeat(DIFFICULTY);
        loop {
            if to_stop.load(Ordering::Relaxed) {
                println!("Receive stop signal for pow");
                return false;
            }

            self.nonce = nonce;
            let digest = self.get_hash();
            if digest.ends_with(&suffix) {
                break;
        }
            nonce += 1;
        }
        return true;
    }

    pub fn is_valid_block(&self, previous_block_hash: &str) -> bool {
        println!("compre previous hash: {} {}", self.previous_block, previous_block_hash);
        if self.previous_block.eq(previous_block_hash) == false {
            println!("Invalid block (bad previous_block)");
            println!("{} {}", self.previous_block, previous_block_hash);
            return false;
        }

        let digest = self.get_hash();
        println!("is_valid_block digest: {}", digest);
        self.print();
        let suffix: String = "0".to_string().repeat(DIFFICULTY);
        if digest.ends_with(&suffix) {
            return true;
        }
        return false;
    }

    pub fn equal(&self, block: &Block) -> bool {
        if self.timestamp == block.timestamp && self.transaction_pool.equal(&block.transaction_pool) && self.previous_block == block.previous_block && self.nonce == block.nonce {
            return true;
        }
        return false;
    }

    pub fn print(&self) {
        println!("print block");
        println!("timestamp: {}", self.timestamp.to_string());
        self.transaction_pool.print();
        println!("previous_block: {}", self.previous_block);
        println!("nonce: {}", self.nonce);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockChain {
    pub chain: Vec<Block>
}

impl BlockChain {
    pub fn create() -> BlockChain {
        println!("Initializing BlockchainManager ...");
        BlockChain {
            chain: Vec::new()
        }
    }

    pub fn clone(&self) -> BlockChain {
        let mut new_chain: Vec<Block> = Vec::new();
        for c in self.chain.iter() {
            new_chain.push(c.clone());
        }
        BlockChain {
            chain: new_chain
        }
    }

    pub fn set_new_block(&mut self, block: Block) {
        self.chain.push(block);
    }

    pub fn renew(&mut self, blockchain: BlockChain) -> (String, bool) {
        if self.is_valid() == true {
            self.chain = blockchain.chain;
            let latest_block = &self.chain[self.chain.len() - 1];
            let latest_block_hash = latest_block.get_hash();
            return (latest_block_hash, true);
        }
        else {
            println!("invalid blockchain cannot be set");
            return ("".to_string(), false);
        }
    }

    pub fn is_valid(&self) -> bool{
        let mut current_index: usize = 1;

        while current_index < self.chain.len() {
            if self.chain[current_index - 1].get_hash() != self.chain[current_index].previous_block {
                return false;
            }
            current_index += 1;
        }
        return true;
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(msg: &str) -> BlockChain {
        serde_json::from_str(msg).unwrap()
    }

    pub fn get_transactions_from_orphan_blocks(&self, orphan_blocks: &BlockChain) -> TransactionPool {
        let mut current_index = 1;
        let mut new_transaction = TransactionPool::create();

        while current_index < orphan_blocks.chain.len() {
            let block = &orphan_blocks.chain[current_index];
            let transactions = &block.transaction_pool;
            let (target, flag) = self.remove_useless_transactions(transactions);
            if flag == true {
                for t in target.transactions {
                    new_transaction.set_new_transaction(t);
                }
            }
            current_index += 1;
        }
        new_transaction
    }

    pub fn remove_useless_transactions(&self, transaction_pool: &TransactionPool) -> (TransactionPool,bool) {
        if transaction_pool.len() != 0 {
            let mut new_transaction_pool = TransactionPool::create();
            let mut current_index: usize= 1;
            while current_index < self.chain.len() {
                let block = &self.chain[current_index];
                let transactions = &block.transaction_pool.transactions;
                
                for t2 in &transaction_pool.transactions {
                    for t in transactions {
                        if t.equal(&t2)  == false {
                            new_transaction_pool.transactions.push(t2.clone());
                        }
                    }
                }
                current_index += 1;
            }
            (new_transaction_pool, true)
        }
        else {
            println!("no transaction to be removed!");
            (TransactionPool::create(), false)
        }
    }

    pub fn resolve_conflicts(&mut self, new_block_chain: BlockChain) -> (BlockChain, BlockChain, bool){
        if new_block_chain.chain.len() > self.chain.len() {
            let mut orphan_blocks = BlockChain::create();
            for b1 in &self.chain {
                let mut contain_flag = false;
                for b2 in &new_block_chain.chain {
                    if b1.equal(b2) == true {
                        contain_flag = true;
                        break;
                    }
                }
                if contain_flag == false {
                    orphan_blocks.set_new_block(b1.clone());
                }
            }
            println!("valid chain is set");
            new_block_chain.print();
            return (new_block_chain, orphan_blocks, true);
        }
        println!("invalid chain cannot be set");
        return (BlockChain::create(), BlockChain::create(), false);
    }

    pub fn get_stored_transactions_from_bc(&self) -> Vec<Transaction> {
        println!("get_stored_transactions_from_bc was called! chain len: {}", self.chain.len());
        let mut current_index: usize = 1;

        let mut stored_trans: Vec<Transaction> = Vec::new();
        while current_index < self.chain.len() {
            let block = &self.chain[current_index];
            current_index += 1;
            for t in &block.transaction_pool.transactions {
                stored_trans.push(t.clone());
            }
        }

        return stored_trans;
    }

    pub fn has_this_output_in_my_chain(&self, transaction_output: &TransactionOutput) -> bool {
        println!("has_this_output_in_my_chain!");

        let mut current_index: usize = 1;

        if self.chain.len() == 1 {
            println!("only the genesis block is in the current chain!");
            return false;
        }

        while current_index < self.chain.len() {
            let block = &self.chain[current_index];
            let transactions = &block.transaction_pool;
            
            for t in transactions.transactions.iter() {
                if t.tx_type == true && t.inputs.len() != 0{ // for coinbase transaction
                    for it in t.inputs.iter() {
                        if it.transaction.outputs[it.output_index].equal(&transaction_output) == true {
                            println!("This transaction was already used!");
                            return true;
                        }
                    }
                }
            }
            current_index += 1;
        }
        return false;
    }

    pub fn is_valid_output_in_my_chain(&self, transaction_output: &TransactionOutput) {
        // to be implemented
    }

    pub fn print(&self) {
        for block in &self.chain {
            block.print();
        }
    }
}

pub fn get_double_sha256(msg: &str) -> String {
    let mut sha256 = Sha256::new();
    sha256.input_str(&msg);
    let res = sha256.result_str();
    let mut sha256_2 = Sha256::new();
    sha256_2.input_str(&res);
    sha256_2.result_str()
}

pub struct UTXOManager {
    pub my_address: String, 
    pub utxo_txs: Vec<(Transaction, usize)>,
    pub my_balance: i64
}

impl UTXOManager {
    pub fn create(my_address: &str) -> UTXOManager {
        UTXOManager {
            my_address: my_address.to_string(), 
            utxo_txs: Vec::new(), 
            my_balance: 0
        }
    }

    pub fn clone(&self) -> UTXOManager {
        let mut new_utxo_txs: Vec<(Transaction, usize)> = Vec::new();
        for u in self.utxo_txs.iter() {
            new_utxo_txs.push((u.0.clone(), u.1));
        }

        UTXOManager { 
            my_address: self.my_address.clone(),
            utxo_txs: new_utxo_txs,
            my_balance: self.my_balance
        }
    }

    pub fn extract_utxo(&mut self, txs: &Vec<Transaction>) {
        println!("extract_utxo called! {}", txs.len());

        let mut outputs: Vec<Transaction> = Vec::new();
        let mut inputs: Vec<Transaction> = Vec::new();

        let mut idx: usize = 0;
        for t in txs.iter() {
            for txout in t.outputs.iter() {
                let recipient = &txout.recipient;
                println!("keys1 {} {}", self.my_address, recipient);
                t.print();
                if self.my_address.eq(recipient) == true {
                    outputs.push(t.clone());
                }
            }

            for txin in t.inputs.iter() {
                let t_in_txin = &txin.transaction;
                idx = txin.output_index;
                let o_recipient = &t_in_txin.outputs[idx].recipient;
                println!("keys2 {} {}", self.my_address, o_recipient);
                t.print();
                if self.my_address.eq(o_recipient) == true{
                    inputs.push(t.clone());
                }
            }
        }

        println!("extracted inputs outputs");
        for t in outputs.iter() {
            t.print();
        }
        for t in inputs.iter() {
            t.print();
        }


        let mut new_outputs: Vec<Transaction> = Vec::new();
        for o in outputs {
            let mut flag: bool = true;
            for i in inputs.iter() {
                for i_i in i.inputs.iter() {
                    if i_i.transaction.equal(&o) == true {
                        flag = false;
                        break;
                    }
                }
            }
            if flag == true {
                new_outputs.push(o)
            }
        }
        self.set_my_utxo_txs(&new_outputs);
    }
    
    pub fn set_my_utxo_txs(&mut self, txs: &Vec<Transaction>) {
        println!("set_my_utxo_txs was called");
        self.utxo_txs.clear();

        for tx in txs {
            println!("tx:");
            tx.print();
            self.put_utxo_tx(&tx);
        }
    }

    pub fn put_utxo_tx(&mut self, tx: &Transaction) {
        println!("put_utxo_tx was called");
        let mut idx: usize = 0;

        for txout in &tx.outputs {
            println!("txout.recipient: {}", txout.recipient);
            println!("self.my_address: {}", self.my_address);
            if self.my_address.eq(&txout.recipient) == true {
                tx.print();
                self.utxo_txs.push((tx.clone(), idx));
            }
            else {
                idx += 1;
            }
        }

        self.compute_my_balance();
    }

    pub fn get_utxo_tx(&mut self, idx: usize) -> &(Transaction, usize){
        return &self.utxo_txs[idx];
    }

    pub fn remove_utxo_tx(&mut self, idx: usize) {
        self.utxo_txs.remove(idx);
        self.compute_my_balance();
    }

    fn compute_my_balance(&mut self)  {
        println!("compute_my_balance was called");

        let mut balance: i64 = 0;
        for t in self.utxo_txs.iter() {
            for txout in t.0.outputs.iter() {
                if self.my_address.eq(&txout.recipient) == true {
                    balance += txout.value;
                }
            }
        }
        self.my_balance = balance;
    }

    pub fn get_txs_to_my_address(&self, txs: &Vec<Transaction>) -> Vec<TransactionOutput> {
        let mut my_txs: Vec<TransactionOutput> = Vec::new();

        for t in txs.iter() {
            for txout in t.outputs.iter() {
                if self.my_address.eq(&txout.recipient) == true {
                    my_txs.push(txout.clone());
                }
            }
        }
        my_txs
    }

    pub fn get_txs_from_my_address(&self, txs: &Vec<Transaction>) -> Vec<Transaction> {
        let mut my_txs: Vec<Transaction> = Vec::new();

        for t in txs.iter() {
            let mut has_my_output = false;
            for txin in t.inputs.iter() {
                let t_in_txin = &txin.transaction;
                let idx = txin.output_index;
                let o_recipient = &t_in_txin.outputs[idx].recipient;
                if self.my_address.eq(o_recipient) == true {
                    has_my_output = true;
                }
            }
            if has_my_output {
                my_txs.push(t.clone());
            }
        }
        //println!("transactions from me: ");
        return my_txs;
    }
}

pub fn run() {
    let mut k_m = keymanager::KeyManager::create(20);
    let mut my_address = k_m.my_address();
    let mut um = UTXOManager::create(&my_address);

    let mut i_k_m = keymanager::KeyManager::create(30);
    let mut u_k_m = keymanager::KeyManager::create(40);

    let mut t1 = Transaction::create_coinbase_transaction(&k_m.my_address(), 30);
    let mut t2 = Transaction::create_coinbase_transaction(&k_m.my_address(), 30);
    let mut t3 = Transaction::create_coinbase_transaction(&k_m.my_address(), 30);

    let mut t4 = Transaction::create();

    let mut t_in = TransactionInput::create(t1.clone(), 0);
    let mut t_out_1 = TransactionOutput::create(&u_k_m.my_address(), 10);
    let mut t_out_2 = TransactionOutput::create(&i_k_m.my_address(), 20);

    t4.inputs.push(t_in);
    t4.outputs.push(t_out_1);
    t4.outputs.push(t_out_2);

    let mut transactions: Vec<Transaction> = Vec::new();
    transactions.push(t1);
    transactions.push(t2);
    transactions.push(t3);
    transactions.push(t4);

    um.extract_utxo(&transactions);

    let balance = um.my_balance;

    println!("my_balance: {}", balance);


}



/*
pub fn generate_block_with_tp(tp: TransactionPool, bc: &mut BlockChain, previous_block_hash: &mut String) -> TransactionPool {
//    thread::spawn(move || loop {
    if tp.transactions.len() == 0 {
        println!("Transaction Pool is empty ...");
        tp
    }
    else {
        let index = tp.transactions.len();
        let new_transaction_pool = tp.clear_my_transactions(index);

        let new_block = Block::create(tp, previous_block_hash.clone());
        let previous_block_hash = new_block.get_hash();
        bc.set_new_block(new_block);
        println!("Current Blockchain is ...");
        bc.print();
        println!("Current previous block hash is {}", previous_block_hash);
        new_transaction_pool
    }
//    });
}
*/
/*
pub fn test() {

    let genesis_block = Block::create_genesis_block();
    let prev_block_hash = genesis_block.get_hash();

    println!("genesis_block_hash {}", prev_block_hash);

    let transaction = Transaction::create(&"test1".to_string(), &"test2".to_string(), 3);

    let new_block = Block::create(transaction, prev_block_hash);
    let new_block_hash = new_block.get_hash();

    println!("new_block_hash: {}", new_block_hash);

    let mut block_chain = BlockChain::create();
    block_chain.set_new_block(genesis_block);
    block_chain.set_new_block(new_block);

    let transaction2 = Transaction::create(&"test1".to_string(), &"test3".to_string(), 2);

    let new_block2 = Block::create(transaction2, new_block_hash);
    block_chain.set_new_block(new_block2);

    block_chain.print();
    println!("is_valid?: {}", block_chain.is_valid());
    
}
*/
