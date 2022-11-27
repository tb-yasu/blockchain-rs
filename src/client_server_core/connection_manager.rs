/*  
    connection_manager.rs
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


pub mod message_manager;

use std::net::{TcpListener, TcpStream};
use std::net::UdpSocket;

use std::collections::HashSet;
use std::str::FromStr;
use std::sync::mpsc;
use std::io::{stdin, Write, BufRead, BufReader};
use std::thread;
use std::time::Duration;

pub struct ConnectionManager {
    my_ip: String, 
    my_port: String, 
    my_c_ip: String, 
    my_c_port: String, 
    core_node_set: HashSet<(String, String)>,
    edge_node_set: HashSet<(String, String)>, 
}

impl ConnectionManager {
    pub fn create(my_ip: &str, my_port: &str, my_c_ip: &str, my_c_port: &str) -> ConnectionManager {
        let mut core_node_set = HashSet::new();
        core_node_set.insert((my_ip.to_string(), my_port.to_string()));

        let edge_node_set = HashSet::new();
        ConnectionManager {
            my_ip: my_ip.to_string(), 
            my_port: my_port.to_string(), 
            my_c_ip: my_c_ip.to_string(), 
            my_c_port: my_c_port.to_string(),
            core_node_set: core_node_set, 
            edge_node_set: edge_node_set
        }
    }

    pub fn clone(&self) -> ConnectionManager {
        ConnectionManager {
            my_ip: self.my_ip.clone(), 
            my_port: self.my_port.clone(),
            my_c_ip: self.my_c_ip.clone(),
            my_c_port: self.my_c_port.clone(),
            core_node_set: self.core_node_set.clone(),
            edge_node_set: self.core_node_set.clone()
        }
    }

    fn add_peer(&mut self, ip: &str, port: &str) {
        //! add a peer with ip and port to the core_node_set.
        println!("add peer: {} {}", ip, port);
        self.core_node_set.insert((ip.to_string(), port.to_string()));
    }

    fn remove_peer(&mut self, ip: &str, port: &str) {
        //! remove a peer with ip and port from the core_node_set.
        if self.core_node_set.contains(&(ip.to_string(), port.to_string())) == true {
            println!("removing peer: {} {}", ip, port);
            self.core_node_set.remove(&(ip.to_string(), port.to_string()));
        }
    }

    fn add_edge_node(&mut self, ip: &str, port: &str) {
        //! set an edge node with ip and port. 
        println!("add edge node: {} {}", ip, port);
        self.edge_node_set.insert((ip.to_string(), port.to_string()));
    }

    fn remove_edge_node(&mut self, ip: &str, port: &str) {
        //! remove an edge node with ip and port.
        if self.edge_node_set.contains(&(ip.to_string(), port.to_string())) == true {
            println!("removing edge node: {} {}", ip, port);
            self.edge_node_set.remove(&(ip.to_string(), port.to_string()));
        }
    }

    pub fn handle_message(&mut self, msg: &str){
        let res = message_manager::parse(&msg);

        println!("received message: {} {} {} {} {} {}", res[0], res[1], res[2], res[3], res[4], res[5]);
        let result: usize    = res[0].parse().unwrap();
        let reason: usize    = res[1].parse().unwrap();
        let cmd: usize       = res[2].parse().unwrap();
        let ip      = &res[3];
        let port    = &res[4];
        let payload = &res[5];

        if      result == message_manager::ERROR && reason == message_manager::ERR_PROTOCOL_UNMATCH {
            println!("Error: Protocol name is not matched");
        }
        else if result == message_manager::ERROR && reason == message_manager::ERR_VERSION_UNMATCH {
            println!("Error: Protocol version is not matched");
        }
        else if result == message_manager::OK && reason == message_manager::OK_WITHOUT_PAYLOAD {
            if cmd == message_manager::MSG_ADD {
                println!("Add node request was received!");
                println!("{} {}", ip, port);
                println!("{} {}", self.my_ip, self.my_port);
                if self.my_ip.eq(ip) == false || self.my_port.eq(port) == false {
                    self.add_peer(&ip, &port);
                    let cl = serde_json::to_string(&self.core_node_set).unwrap();
                    let msg = message_manager::build(message_manager::MSG_CORE_LIST, &self.my_ip, &self.my_port, &&cl);
                    println!("{}", msg);
                    self.send_to_all_peer(&msg);
                }
            }
            else if cmd == message_manager::MSG_REMOVE {
                println!("Remove request was received from {} {}", ip, port);
                self.remove_peer(&ip, &port);
                let cl = serde_json::to_string(&self.core_node_set).unwrap();
                let msg = message_manager::build(message_manager::MSG_CORE_LIST, &self.my_ip, &self.my_port, &cl);
                self.send_to_all_peer(&msg);
            }
            else if cmd == message_manager::MSG_PING {
                println!("MSG_PING pass");
            }
            else if cmd == message_manager::MSG_REQUEST_CORE_LIST {
                println!("List for Core nodes was requested!");
                let cl = serde_json::to_string(&self.core_node_set).unwrap();
                let msg = message_manager::build(message_manager::MSG_CORE_LIST, &self.my_ip, &self.my_port, &cl);
                send_msg(&ip, &port, &msg);
            }
            else if cmd == message_manager::MSG_ADD_AS_EDGE {
                self.add_edge_node(&ip.to_string(), &port.to_string());
                let cl = serde_json::to_string(&self.core_node_set).unwrap();
                let msg = message_manager::build(message_manager::MSG_CORE_LIST, &self.my_ip, &self.my_port, &cl);
                send_msg(&ip, &port, &msg);
            }
            else if cmd == message_manager::MSG_REMOVE_EDGE {
                self.remove_edge_node(&ip, &port);
            }
            else {
   //             println!("received unknown command {}", cmd);
            }
        }
        else if result == message_manager::OK && reason == message_manager::OK_WITH_PAYLOAD {
            if cmd == message_manager::MSG_CORE_LIST {
                println!("Refresh the core node list!");

                let res = serde_json::from_str(&payload);
                match res {
                    Ok(result) => self.core_node_set = result,
                    Err(msg) => println!("failure {}", msg),
                }

                println!("new core node list");
                for p in &self.core_node_set {
                    println!("{} {}", p.0, p.1);
                }
            }
            else {
                println!("received unknown command {}", cmd);
            }
        }
        else {
            println!("Unexpected status");
        }
    }

    pub fn send_to_all_peer(&self, msg: &str) {
        println!("send_to_all_peer was called!");

        for core_addr in self.core_node_set.iter() {
            if self.my_ip.eq(&core_addr.0) == false || self.my_port.eq(&core_addr.1) == false {
                let server_addr = core_addr.0.clone() + &":" + &core_addr.1;
                let mut socket = TcpStream::connect(&server_addr).expect("cannot connect to the server");
                socket.set_nonblocking(true).expect("unable");
                println!("connected to {}", &server_addr);
    
                let bytes = String::from(msg).into_bytes();
                if let Err(e) = socket.write_all(&bytes) {
                    println!("Send error: {}", e);
                    continue;
                }
            }
        }
    }

    pub fn is_core(&self, ip: &str, port: &str) -> bool {
        self.core_node_set.contains(&(ip.to_string(), port.to_string()))
    }

    pub fn check_peers_connection(&mut self, tx: mpsc::Sender<String>) {
        println!("check_peers_connection");
        
        let mut new_core_node_set: HashSet<(String, String)> = HashSet::new();
        for core in self.core_node_set.iter() {
            println!("ping to {} {}", core.0, core.1);
            if ((self.my_ip.eq(&core.0) == true && self.my_port.eq(&core.1) == true)) || is_alive(&core.0, &core.1) == true {
                new_core_node_set.insert((core.0.clone(), core.1.clone()));
            }
        }
        if self.core_node_set.len() != new_core_node_set.len() {
            self.core_node_set = new_core_node_set;
            let cl = serde_json::to_string(&self.core_node_set).unwrap();
            let msg = get_message_text(message_manager::MSG_CORE_LIST, &self.my_ip, &self.my_port, &cl);
            self.send_to_all_peer(&msg);
        }
    }
}

pub fn send_msg(ip: &str, port: &str, msg: &str) {
    let server_addr = ip.to_string() + &":" + port;
    let mut socket = TcpStream::connect(&server_addr).expect("cannot connect to the server");
    socket.set_nonblocking(true).expect("unable");
    println!("connected to {}", &server_addr);

    let bytes = String::from(msg).into_bytes();
    if let Err(e) = socket.write_all(&bytes) {
        println!("Send error: {}", e);
    }
}

pub fn join_network(my_ip: &str, my_port: &str, my_c_ip: &str, my_c_port: &str) {
    println!("join_neftwork {} {}", my_c_ip, my_c_port);
    if my_c_ip.len() > 0 && my_c_port.len() > 0 {
       connect_to_p2pnw(my_ip, my_port, my_c_ip, my_c_port);
   }
}

fn connect_to_p2pnw(my_ip: &str, my_port: &str, host: &str, port: &str) {
    let server_addr = host.to_string() + &":" + &port;
    let mut socket = TcpStream::connect(&server_addr).expect("cannot connect to the server");
    socket.set_nonblocking(true).expect("unable");
    let msg = message_manager::build(message_manager::MSG_ADD, my_ip, my_port, &"".to_string());
    println!("connected to {}", &server_addr);

    let bytes = String::from(msg).into_bytes();
    if let Err(e) = socket.write_all(&bytes) {
        println!("Send error: {}", e);
    }
}

fn is_alive(ip: &str, port: &str) -> bool {
    println!("is_alive {} {}", ip, port);
    let server_addr = ip.to_string() + &":" + port;

    let res = TcpStream::connect(&server_addr);
    match res {
        Ok(socket) => { return true; },
        Err(E) => { return false;}
    }
}

pub fn receiver(client: TcpStream, tx: mpsc::Sender<String>) {
    let mut reader = BufReader::new(client);

    let mut msg = String::new();
    if let Ok(n) = reader.read_line(&mut msg) {
        if n > 0 { 
            println!("received: {}", msg);
            tx.send(msg).unwrap();
        }
    }
}

pub fn get_message_text(msg_type: usize, ip: &str, port: &str, payload: &str) -> String {
    let msgtxt = message_manager::build(msg_type, ip, port, payload);
    println!("msgtxt: {}", msgtxt);
    msgtxt
}

pub struct ConnectionManager4Edge {
   pub ip: String, 
   pub port: String, 
   pub my_c_ip: String, 
   pub my_c_port: String, 
   pub core_node_set: HashSet<(String, String)>
}

impl ConnectionManager4Edge {
    pub fn create(ip: &str, port: &str, my_c_ip: &str, my_c_port: &str) -> ConnectionManager4Edge {
        let mut core_node_set = HashSet::new();
        core_node_set.insert((ip.to_string(), port.to_string()));
    
        ConnectionManager4Edge {
            ip: ip.to_string(),
            port: port.to_string(), 
            my_c_ip: my_c_ip.to_string(), 
            my_c_port: my_c_port.to_string(),
            core_node_set: core_node_set
        }
    }

    pub fn clone(&self) -> ConnectionManager4Edge{
        ConnectionManager4Edge {
            ip: self.ip.clone(),
            port: self.port.clone(),
            my_c_ip: self.my_c_ip.clone(),
            my_c_port: self.my_c_port.clone(),
            core_node_set: self.core_node_set.clone()
        }
    }

    pub fn connection_close(&self) {
        // to be implemented
    }

    pub fn send_msg(&mut self, msg: &str) {
        println!("Sending... {}", msg);
        let server_addr = self.my_c_ip.clone() + &":" + &self.my_c_port;
        let mut socket = TcpStream::connect(&server_addr).expect("cannot connect to the server");
        socket.set_nonblocking(true).expect("unable");
        let bytes = String::from(msg).into_bytes();

        println!("Trying to connect into P2P network ...");
        if let Err(e) = socket.write_all(&bytes) {
            if self.core_node_set.len() != 0 {
                for x in self.core_node_set.iter() {
                    if x.0 != self.my_c_ip || x.1 != self.my_c_port {
                        self.my_c_ip = x.0.clone();
                        self.my_c_port = x.1.clone();
                        break;
                   }
                }
                self.send_msg(msg);
            }
            else {
                println!("No core node found in our list ...");
            }
        }
    }

    pub fn handle_message(&mut self, msg: &str) {
        let res = message_manager::parse(&msg);
    
        println!("received message: {} {} {} {} {} {}", res[0], res[1], res[2], res[3], res[4], res[5]);
        let result: usize    = res[0].parse().unwrap();
        let reason: usize    = res[1].parse().unwrap();
        let cmd: usize       = res[2].parse().unwrap();
        let ip      = &res[3];
        let port    = &res[4];
        let payload = &res[5];
    
        if      result == message_manager::ERROR && reason == message_manager::ERR_PROTOCOL_UNMATCH {
            println!("Error: Protocol name is not matched");
        }
        else if result == message_manager::ERROR && reason == message_manager::ERR_VERSION_UNMATCH {
            println!("Error: Protocol version is not matched");
        }
        else if result == message_manager::OK && reason == message_manager::OK_WITHOUT_PAYLOAD {
            if cmd == message_manager::MSG_PING {
                // pass
            }
            else {
                println!("Edge does not have functions for this message!");
            }
        }
        else if result == message_manager::OK && reason == message_manager::OK_WITH_PAYLOAD {
            if cmd == message_manager::MSG_CORE_LIST {
                println!("Refresh the core node list!");
                self.core_node_set = serde_json::from_str(&payload).unwrap();

                println!("new core node list");
                for p in &self.core_node_set {
                    println!("{} {}", p.0, p.1);
                }
            }
            else {
                println!("received unknown command {}", cmd);
            }
        }
        else {
            println!("Unexpected status");
        }
    }

    pub fn send_ping(&mut self) {
        println!("send ping from edge node!");
        if is_alive(&self.my_c_ip, &self.my_c_port) == false {
            if self.core_node_set.len() == 0 {
                println!("No core node found in our list");
                return;
            }
            self.core_node_set.remove(&(self.my_c_ip.clone(), self.my_c_port.clone()));
            for p in &self.core_node_set {
                if self.my_c_ip != p.0 || self.my_c_port != p.1 {
                    self.my_c_ip   = p.0.clone();
                    self.my_c_port = p.1.clone();
                    break;
                }
            }
        }
    }

    pub fn connect_to_core_node_4edge(&self) {
        self.connect_to_p2pnw_4edge();
    }
    
    fn connect_to_p2pnw_4edge(&self) {
        let server_addr = self.my_c_ip.to_string() + &":" + &self.my_c_port.to_string();
        let mut socket = TcpStream::connect(&server_addr).expect("cannot connect to the server");
        socket.set_nonblocking(true).expect("unable");
        let msg = message_manager::build(message_manager::MSG_ADD_AS_EDGE, &self.ip, &self.port, &"".to_string());
        println!("connected to {}", &server_addr);
    
        let bytes = String::from(msg).into_bytes();
        if let Err(e) = socket.write_all(&bytes) {
            println!("Send error: {}", e);
        }
    }

    pub fn receiver_4edge(client: TcpStream, tx: mpsc::Sender<String>) {
        let mut reader = BufReader::new(client);
    
        thread::spawn(move || {
            let mut msg = String::new();
            if let Ok(n) = reader.read_line(&mut msg) {
                if n > 0 { 
                    println!("{}", msg);
                    tx.send(msg).unwrap();
                }
            }
            thread::sleep(Duration::from_millis(100));
        });
    }
}


