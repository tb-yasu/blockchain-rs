/*  
    client_server_core.rs
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

pub mod connection_manager;
pub mod block_chain;

use crate::client_server_core;
use crate::client_server_core::block_chain::Transaction;
use crate::client_server_core::block_chain::TransactionPool;
use crate::client_server_core::block_chain::Block;
use crate::client_server_core::block_chain::BlockChain;
use crate::client_server_core::connection_manager::ConnectionManager;
use crate::client_server_core::connection_manager::message_manager;

use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::net::{TcpListener, TcpStream};
use std::net::UdpSocket;
use std::collections::HashSet;
use std::time::SystemTime;
use chrono::Local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use self::block_chain::UTXOManager;
use self::connection_manager::ConnectionManager4Edge;

extern crate minisign;
use minisign::{KeyPair, PublicKeyBox, SecretKeyBox, SignatureBox};
use openssl::sign;
use std::io::Cursor;

extern crate rand;
use rand::Rng;

pub const STATE_INIT: u64 = 0;
pub const STATE_STANDBY: u64 = 1;
pub const STATE_CONNECTED_TO_NETWORK: u64 = 2;
pub const STATE_SHUTTING_DOWN: u64 = 3;

pub const STATE_INIT_4EDGE: u64 = 0;
pub const STATE_ACTIVE_4EDGE: u64 = 1;
pub const STATE_SHUTTING_DOWN_4EDGE: u64 = 2;

/// The time interval for generating a block with a current transaction pool
pub const MINING_INTERVAL: i64 = 60;
/// The time interval for checking peers connections
pub const CHECK_PEERS_CONNECTION_INTERVAL: i64 = 20;
pub struct Worker {
    to_stop: Arc<AtomicBool>,
}

trait WorkerTrait {
    fn run(&self);
    fn stop(&self);
}

impl Worker {
    fn new() -> Worker {
        Worker {
            to_stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub struct ServerCore {
    server_state: u64,
    my_ip: String,
    my_port: String, 
    core_node_ip: String, 
    core_node_port: String,
    connection_manager: ConnectionManager,
    tp: TransactionPool,
    bc: BlockChain,
    previous_block_hash: String, 
    km: block_chain::keymanager::KeyManager, 
    um: UTXOManager
}

impl ServerCore {
    pub fn create(my_ip: &str, my_port: &str, core_node_ip: &str, core_node_port: &str) -> ServerCore {
        println!("Initializing server...");
        println!("Server IP address is set to {}", my_ip);

        let gc = Block::create_genesis_block();
        let gc_hash = gc.get_hash();
        println!("initial hash value: {}", gc_hash);
        let mut bc = BlockChain::create();
        bc.set_new_block(gc);

        let mut rng = rand::thread_rng();
        let rand_num = rng.gen::<usize>();
        let km = client_server_core::block_chain::keymanager::KeyManager::create(rand_num);
        let my_address = km.my_address();
        let um = UTXOManager::create(&my_address);

        ServerCore {
            server_state: STATE_INIT, 
            my_ip: my_ip.to_string(), 
            my_port: my_port.to_string(), 
            core_node_ip: core_node_ip.to_string(), 
            core_node_port: core_node_port.to_string(),
            connection_manager: ConnectionManager::create(my_ip, my_port, &core_node_ip, &core_node_port),
            tp: TransactionPool::create(),
            bc: bc,
            previous_block_hash: gc_hash,
            km: km, 
            um: um
        }
    }

    pub fn clone(&self) -> ServerCore {
        println!("Cloning server...");

        ServerCore {
            server_state: self.server_state.clone(), 
            my_ip: self.my_ip.clone(),
            my_port: self.my_port.clone(),
            core_node_ip: self.core_node_ip.clone(),
            core_node_port: self.core_node_port.clone(),
            connection_manager: self.connection_manager.clone(),
            tp: self.tp.clone(),
            bc: self.bc.clone(),
            previous_block_hash: self.previous_block_hash.clone(),
            km: self.km.clone(),
            um: self.um.clone()
        }
    }

    pub fn start(&mut self) {
        println!("start");
        self.server_state = STATE_STANDBY;
        self.wait_for_access();

    }

    pub fn join_network(&mut self) {
        println!("start_join_network");
        self.server_state = STATE_CONNECTED_TO_NETWORK;
        connection_manager::join_network(&self.my_ip, &self.my_port, &self.core_node_ip, &self.core_node_port);
    }

    pub fn shutdown(&mut self) {
        self.server_state = STATE_SHUTTING_DOWN;
        println!("Shutdown server...");
    }

    pub fn get_my_current_state(&mut self) -> u64 {
        self.server_state
    }

    fn handle_message(&mut self, msg: &str, locked: &mut bool, cur_time: &mut i64, w: &Worker) {
        let res = connection_manager::message_manager::parse(&msg);

        println!("received message: {} {} {} {} {} {}", res[0], res[1], res[2], res[3], res[4], res[5]);

        let cmd: usize       = res[2].parse().unwrap();
        let ip      = &res[3];
        let port    = &res[4];
        let payload = &res[5];
        
        println!("received msg cmd ip port payload {} {} {} {}", cmd, ip, port, payload);

        if cmd == connection_manager::message_manager::MSG_REQUEST_FULL_CHAIN {
            println!("Send our latest blockchain to : {}:{}", ip, port);
            let bc_str = self.bc.to_string();
            let new_msg = connection_manager::get_message_text(connection_manager::message_manager::RSP_FULL_CHAIN, &self.my_ip, &self.my_port, &bc_str);
            connection_manager::send_msg(&ip, &port, &new_msg);
        }
        else if cmd == connection_manager::message_manager::MSG_NEW_TRANSACTION {
            let new_transaction: Transaction = Transaction::from_str(&payload);

            println!("received transaction: ");
            new_transaction.print();

            if self.tp.contain(&new_transaction) {
                println!("this has already been pooled transaction!");
                return;
            }

            if self.connection_manager.is_core(&ip, &port) == false {
                self.tp.push(new_transaction);
                println!("current transaction pool!");
                self.tp.print();
                let new_msg = connection_manager::get_message_text(connection_manager::message_manager::MSG_NEW_TRANSACTION, &self.my_ip, &self.my_port, payload);
                self.connection_manager.send_to_all_peer(&new_msg);
            }
            else if self.tp.contain(&new_transaction) == false {
                self.tp.push(new_transaction);
                println!("current transaction pool!");
                self.tp.print();
            }
        }
        else if cmd == connection_manager::message_manager::MSG_NEW_BLOCK {
            println!("MSG_NEW_BLOCK");
            // if sender and receiver have the same ip and port, do nothing!
            if self.my_ip.eq(ip) == true && self.my_port.eq(port) == true {
                println!("sender and receiver have the same ip and port");
                return;
            }
            if self.connection_manager.is_core(&ip, &port) == false {
                println!("block received from unknown core node");
                return;
            }

            w.to_stop.store(true, Ordering::Relaxed);

            let new_block = Block::from_string(&payload);
            new_block.print();

            if new_block.is_valid_block(&self.previous_block_hash) == true {
                println!("valid block is received and blockchain is updated");
                self.previous_block_hash = new_block.get_hash();
                self.bc.set_new_block(new_block);
            }
            else {
                // request all chains for resolving conflicts
                println!("received blockchain is invalid. full chain is requested.");
                let msg = connection_manager::get_message_text(connection_manager::message_manager::MSG_REQUEST_FULL_CHAIN, &self.my_ip, &self.my_port, "");
                self.connection_manager.send_to_all_peer(&msg);
            }
        }
        else if cmd == connection_manager::message_manager::MSG_NEW_BLOCK_TO_ALL {
            if self.connection_manager.is_core(&ip, &port) == false {
                println!("block received from unknown core node");
                return;
            }

            println!("MSG_NEW_BLOCK_TO_ALL");
            w.to_stop.store(true, Ordering::Relaxed);

            println!("msg payload: {}", payload);
            let new_block = Block::from_string(&payload);
            new_block.print();

            if new_block.is_valid_block(&self.previous_block_hash) == true {
                println!("valid block is received and blockchain is updated");
                self.previous_block_hash = new_block.get_hash();
                self.bc.set_new_block(new_block);
                let msg = connection_manager::get_message_text(connection_manager::message_manager::MSG_NEW_BLOCK, &ip, &port, &payload);
                self.connection_manager.send_to_all_peer(&msg);
            }
            else {
                // request all chains for resolving conflicts
                println!("received blockchain is invalid. full chain is requested.");
                let msg = connection_manager::get_message_text(connection_manager::message_manager::MSG_REQUEST_FULL_CHAIN, &self.my_ip, &self.my_port, "");
                self.connection_manager.send_to_all_peer(&msg);
            }
        }
        else if cmd == connection_manager::message_manager::RSP_FULL_CHAIN {
            if self.connection_manager.is_core(&ip, &port) == false {
                println!("block received from unknown core node");
                return;
            }
            let new_block_chain = BlockChain::from_string(&payload);
            let (new_bc, orphan_blocks, valid_flag) = self.bc.resolve_conflicts(new_block_chain);
            if valid_flag == true {
                let (previous_block_hash, flag)= self.bc.renew(new_bc);
                if flag == true {
                    self.previous_block_hash = previous_block_hash;
                    if orphan_blocks.chain.len() != 0 {
                        let new_transactions = self.bc.get_transactions_from_orphan_blocks(&orphan_blocks);

                        for t in new_transactions.transactions {
                            self.tp.set_new_transaction(t);
                        }
                    }
                }
            }
            else {
                println!("Received transaction is useless!");
            }
        }
        else if cmd == connection_manager::message_manager::MSG_ENHANCED {
            // pass
        }
        else if cmd == connection_manager::message_manager::MSG_UNLOCKED {
            *locked = false;
            *cur_time = Local::now().timestamp();
        }
        else if cmd == connection_manager::message_manager::MSG_SENDMSGALLPEAR {
            println!("send_msg_to_all_pear1: {}", payload);
            self.connection_manager.send_to_all_peer(&payload);
        }
    }

    fn check_availability_of_transaction(&self, transaction: &Transaction) -> bool {
        let (result, used_outputs) = verify_sbc_transaction_sig(&transaction);

        if result == false {
            println!("signature verification error on new transaction");
            return false;
        }

        for used_o in used_outputs.iter() { 
            let bv_v_result = self.bc.has_this_output_in_my_chain(used_o);

            if bv_v_result == true {
                println!("This transactionOutput is already used");
                return false;
            }
        }
        return true;
    }

    pub fn generate_block_with_tp(transaction_pool: TransactionPool, km: block_chain::keymanager::KeyManager, previous_block_hash: String, my_ip: String, my_port: String, tx: mpsc::Sender<String>, to_stop: &Arc<AtomicBool>) {
        //! It generates a block with a transaction pool by the proof of work algorithm.

        if transaction_pool.len() == 0 {
            return;
        }

        let mut total_fee = transaction_pool.get_total_fee_from_tp();
        total_fee += 30;

        let my_coinbase_t = Transaction::create_coinbase_transaction(&km.public_key_str, total_fee);

        let mut new_transaction_pool = transaction_pool.clone();
        new_transaction_pool.set_new_transaction(my_coinbase_t);
        let mut new_block = Block::create(new_transaction_pool, previous_block_hash);
        let flag = new_block.compute_nonce_for_pow(to_stop);
        if flag == true {
            let new_block_string = new_block.to_string();
            let msg = connection_manager::get_message_text(connection_manager::message_manager::MSG_NEW_BLOCK_TO_ALL, &my_ip, &my_port, &new_block_string);
            println!("created transaction msg: {}", msg);
            tx.send(msg).unwrap();  
        }
        let cur_time = Local::now().timestamp();
        let msg2 = connection_manager::get_message_text(connection_manager::message_manager::MSG_UNLOCKED, &my_ip, &my_port, &cur_time.to_string());
        tx.send(msg2).unwrap();
    }

    pub fn get_total_fee_on_block(&self, block: &Block) -> i64 {
        println!("get_total_fee_on_block is called!");

        let transactions = &block.transaction_pool.transactions;
        let mut result = 0;
        for t in transactions.iter() {
            let mut total_in = 0;
            for i in t.inputs.iter() {
                total_in += i.transaction.outputs[i.output_index].value;
            }
            let mut total_out = 0;
            for o in t.outputs.iter() {
                total_out += o.value;
            }
            let delta = total_in - total_out;
            result += delta;
        }
        return result;
    }

    pub fn check_transactions_in_new_block(&self, block: &Block) -> bool {
        let mut fee_for_block: i64 = self.get_total_fee_on_block(block);
        fee_for_block += 30;
        
        println!("fee_for_block: {}", fee_for_block);

        let transactions = &block.transaction_pool.transactions;

        let mut counter = 0;

        for t in transactions.iter() {
            if t.tx_type == false { // for general transaction
                let flag = self.check_availability_of_transaction(t);
                if flag == false {
                    println!("Bad block. Having invalid Transaction");
                    return false;
                }
            }
            else { // for coinbase transaction
                if counter != 0 {
                    println!("Coinbase Transaction is only for BlockBuilder!");
                    return false;
                }
                else {
                    let insentive = t.outputs[0].value;
                    println!("insentive: {}", insentive);
                    if insentive != fee_for_block {
                        println!("Invalid value in fee for Coinbase Transaction {}", insentive);
                        return false;
                    }
                }
            }
        }
        println!("ok. this block is acceptable.");
        return true;
    }

    pub fn check_availability_of_transaction_in_block(&self, transaction: &Transaction) -> bool {
        let (result, used_outputs) = verify_sbc_transaction_sig(transaction);

        if result == false {
            println!("signature verification error on new transaction");
            return false;
        }

        for used_o in used_outputs.iter() {
            println!("used outputs: ");
            used_o.print();

            let bm_v_result = self.bc.has_this_output_in_my_chain(used_o);
//            let bm_v_result2 = self.bc.is_valid

            if bm_v_result == false {
                println!("This transaction output is unknown.");
                return false;
            }
        }
        return true;
    }

    fn wait_for_access(&self) {
        println!("execute wait_for_access");
        let addr = self.my_ip.to_string() + ":" + &self.my_port;

        println!("starting server at {}", addr);
        let server = TcpListener::bind(addr).expect("Faiulre in starting server");
        server.set_nonblocking(true).expect("unusable");

        let (tx, rx) = mpsc::channel::<String>();

        let mut server_core = self.clone();

        let mut cur_time = Local::now().timestamp();
        let mut locked: bool = false; 
        let w = Worker::new();
        let mut iter = 0;
        thread::spawn(move || loop {
            server_core.bc.print();
            if let Ok((client, addr)) = server.accept() {
                println!("Connected by {}", addr);
                connection_manager::receiver(client, tx.clone());
            }
            if let Ok(msg) = rx.try_recv() {
                println!("received msg: {}", msg);
                server_core.connection_manager.handle_message(&msg);
                server_core.handle_message(&msg, &mut locked, &mut cur_time, &w);
            }
            if locked == false && Local::now().timestamp() - cur_time > MINING_INTERVAL{
                let transaction_pool_clone = server_core.tp.clone();
                let km_clone = server_core.km.clone();
                server_core.tp.clear();
                let previous_block_hash_clone = server_core.previous_block_hash.clone();
                let my_ip_tmp = server_core.my_ip.clone();
                let my_port_tmp = server_core.my_port.clone();
                let tx_clone = tx.clone();
                cur_time = Local::now().timestamp();
                let to_stop = Arc::clone(&w.to_stop);

                thread::spawn(move || {
                    ServerCore::generate_block_with_tp(transaction_pool_clone, km_clone, previous_block_hash_clone, my_ip_tmp, my_port_tmp, tx_clone, &to_stop);
                });
            }
            if iter == CHECK_PEERS_CONNECTION_INTERVAL {
                server_core.connection_manager.check_peers_connection(tx.clone());
                iter = 0;
            }
            iter += 1;

            thread::sleep(Duration::from_millis(1000));
        });
    }
}

pub fn verify_signature(message: &str, signature_box_str: &str, sender_public_key_box_str: &str) -> bool {
    let signature_box = SignatureBox::from_string(signature_box_str).unwrap();
       
    let pk_box = PublicKeyBox::from_string(&sender_public_key_box_str).unwrap();
    let pk = pk_box.into_public_key().unwrap();

    let msg_reader = Cursor::new(message);
    let verified = minisign::verify(&pk, &signature_box, msg_reader, true, false, false);

    let flag: bool;
    match verified {
        Ok(()) => flag = true,
        Err(_) => flag = false,
    };
    return flag;
}

pub fn verify_sbc_transaction_sig(transaction: &Transaction) -> (bool, Vec<block_chain::TransactionOutput>){
    println!("verify_sbc_transaction_sig was called");

    let (sender_pubkey_text, used_outputs) = get_pubkey_from_sbc_transaction(transaction);
    let signature = &transaction.signature;
    let mut c_transaction = transaction.clone();
    c_transaction.signature = "".to_string();
    let target_txt = c_transaction.to_string();

    let result = verify_signature(&target_txt, &signature, &sender_pubkey_text);

    return (result, used_outputs);
}

fn get_pubkey_from_sbc_transaction(transaction: &Transaction) -> (String, Vec<block_chain::TransactionOutput>){
    println!("get_public_key_from_sbc_transaction was called");

    let input_t_list = &transaction.inputs;
    let mut used_output: Vec<block_chain::TransactionOutput> = Vec::new();
    let mut sender_pubkey: &String = &"".to_string();
    for i in input_t_list.iter() {
        let idx = i.output_index;        
        let tx = &i.transaction.outputs[idx];
        used_output.push(tx.clone());
        sender_pubkey = &tx.recipient;
    }

    (sender_pubkey.clone(), used_output)
}

pub struct ClientCore {
    pub client_state: u64,
    pub my_ip: String,
    pub my_port: String, 
    pub core_node_ip: String, 
    pub core_node_port: String, 
    pub bc: BlockChain,
    pub prev_block_hash: String, 
    pub cm: ConnectionManager4Edge
}

impl ClientCore {
    pub fn create(my_ip: &str, my_port: &str, core_node_ip: &str, core_node_port: &str) -> Self {
        println!("Initializing server...");
        println!("Server IP address is set to {}", my_ip);

        ClientCore {
            client_state: STATE_INIT_4EDGE, 
            my_ip: my_ip.to_string(), 
            my_port: my_port.to_string(), 
            core_node_ip: core_node_ip.to_string(), 
            core_node_port: core_node_port.to_string(),
            bc: BlockChain::create(),
            prev_block_hash: String::new(),
            cm: ConnectionManager4Edge::create(my_ip, my_port, core_node_ip, core_node_port)
        }
    }

    pub fn clone(&self) -> ClientCore {
        ClientCore {
            client_state: self.client_state.clone(),
            my_ip: self.my_ip.clone(),
            my_port: self.my_port.clone(),
            core_node_ip: self.core_node_ip.clone(),
            core_node_port: self.core_node_port.clone(),
            bc: self.bc.clone(),
            prev_block_hash: self.prev_block_hash.clone(),
            cm: self.cm.clone()
        }
    }

    pub fn start(&mut self, tx: &mpsc::Sender<String>) {
        println!("Start edge node ....");
        self.client_state = STATE_ACTIVE_4EDGE;
        self.wait_for_access_4edge(tx.clone());
        self.cm.connect_to_core_node_4edge();
    }

    pub fn shutdown(&mut self) {
        self.client_state = STATE_SHUTTING_DOWN_4EDGE;
        println!("Shutdown edge node ...");
    }

    pub fn get_my_current_state(&self) -> u64 {
        self.client_state
    }

    pub fn send_message_to_my_core_node(&mut self, msg_type: usize, msg: &str) {
        let msg = connection_manager::get_message_text(msg_type, &self.core_node_ip, &self.core_node_port, &msg);
        println!("msgtxt: {}", msg);
        self.cm.send_msg(&msg);
    }

    pub fn send_req_full_chain_to_my_core_node(&mut self) {
        println!("send a request of the full chain to my core node");
        let new_message = connection_manager::get_message_text(connection_manager::message_manager::MSG_REQUEST_FULL_CHAIN, &self.my_ip, &self.my_port, "");
        self.cm.send_msg(&new_message);
    }

    pub fn update_callback(&self) {
        println!("update callback was called!");
//        let s_transactions = self.
    }

    pub fn handle_message(&mut self, msg: &str, tx: mpsc::Sender<String>)  {
        let res = connection_manager::message_manager::parse(&msg);

        println!("received message: {} {} {} {} {} {}", res[0], res[1], res[2], res[3], res[4], res[5]);


        let cmd: usize       = res[2].parse().unwrap();
        let ip      = &res[3];
        let port    = &res[4];
        let payload = &res[5];
        
        if cmd == message_manager::RSP_FULL_CHAIN {
            let new_block_chain: BlockChain = serde_json::from_str(payload).unwrap();
            let (new_block_chain, ortphan_blocks, flag) = self.bc.resolve_conflicts(new_block_chain);
            println!("blockchain received from central");
            new_block_chain.print();
            if flag == true {
                println!("obtained bc");
                new_block_chain.print();
                self.bc = new_block_chain;
                let msg = serde_json::to_string(&self.bc).unwrap();
                tx.send(msg).unwrap();
            }
            else {
                println!("Received blockchain is useless");
            }
        }

    }

    pub fn wait_for_access_4edge(&mut self, tx_main: mpsc::Sender<String>) {
        println!("execute __wait_for_access");

        let mut client_core = client_server_core::ClientCore::create(&self.my_ip, &self.my_port, &self.core_node_ip, &self.core_node_port);

        let addr = self.cm.ip.to_string() + ":" + &self.cm.port.to_string();
        println!("starting server at {}", addr);
        let server = TcpListener::bind(addr).expect("Faiulre in starting server");
        server.set_nonblocking(true).expect("unusable");
        let (tx, rx) = mpsc::channel::<String>();
        let mut iter = 0;

        thread::spawn(move || loop {
            if let Ok((client, addr)) = server.accept() {
                println!("Connected by {}", addr);
                connection_manager::ConnectionManager4Edge::receiver_4edge(client, tx.clone());
            }
            if let Ok(msg) = rx.try_recv() {
                println!("received msg: {}", msg.trim());
                client_core.cm.handle_message(&msg);
                client_core.handle_message(&msg, tx_main.clone());
            }
            if iter == 20 {
                client_core.cm.send_ping();
                iter = 0;
            }
            iter += 1;
            thread::sleep(Duration::from_millis(1000));
        });
    }
}