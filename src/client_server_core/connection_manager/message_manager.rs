/*  
    message_manager.rs
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

use serde::{Serialize, Deserialize};

pub const PROTOCOL_NAME: &str = "blockchain-rs_protocol";
pub const MY_VERSION: &str = "0.1.0";

pub const MSG_ADD: usize = 0;
pub const MSG_REMOVE: usize = 1;
pub const MSG_CORE_LIST: usize = 2;
pub const MSG_REQUEST_CORE_LIST: usize = 3;
pub const MSG_PING: usize = 4;
pub const MSG_ADD_AS_EDGE: usize = 5;
pub const MSG_REMOVE_EDGE: usize = 6;
pub const MSG_NEW_TRANSACTION: usize = 7;
pub const MSG_NEW_BLOCK: usize = 8;
pub const MSG_NEW_BLOCK_TO_ALL: usize = 9;
pub const MSG_REQUEST_FULL_CHAIN: usize = 10;
pub const RSP_FULL_CHAIN: usize = 11;
pub const MSG_ENHANCED: usize = 12;
pub const MSG_UNLOCKED: usize = 13;
pub const MSG_SENDMSGALLPEAR: usize = 14;

pub const ERR_PROTOCOL_UNMATCH: usize = 0;
pub const ERR_VERSION_UNMATCH: usize = 1;
pub const OK_WITH_PAYLOAD: usize = 2;
pub const OK_WITHOUT_PAYLOAD: usize = 3;

pub const ERROR: usize = 0;
pub const OK: usize = 1;
pub const NONE: usize = 0;


#[derive(Serialize, Deserialize, Debug)]
pub struct MessageManager {
    pub protocol: String, 
    pub version: String, 
    pub msg_type: usize, 
    pub ip: String, 
    pub port: String,
    pub payload: String
}

pub fn build(msg_type: usize, ip: &str, port: &str, payload: &str) -> String {
//! It builds a message in String format with a given msg_type, ip, port and payload.    

    let mm = MessageManager {
        protocol: PROTOCOL_NAME.to_string(), 
        version: MY_VERSION.to_string(),  
        msg_type: msg_type,
        ip: ip.to_string(), 
        port: port.to_string(), 
        payload: payload.to_string()          
    };

    serde_json::to_string(&mm).unwrap()
}

pub fn parse(msg: &str) -> Vec<String> {
//! It parses a message build by the build function.

    let mm: MessageManager = serde_json::from_str(msg).unwrap();

    println!("parsed message: {} {} {} {} {}", mm.version, mm.msg_type, mm.ip, mm.port, mm.payload);
    println!("payload: {}", mm.payload);

    let mut res: Vec<String> = Vec::new();
    if mm.protocol != PROTOCOL_NAME {
        res = vec![ERROR.to_string(), ERR_PROTOCOL_UNMATCH.to_string(), NONE.to_string(), NONE.to_string(), NONE.to_string(), NONE.to_string()];
    }
    else if mm.version != MY_VERSION {
        res = vec![ERROR.to_string(), ERR_VERSION_UNMATCH.to_string(), NONE.to_string(), NONE.to_string(), NONE.to_string(), NONE.to_string()];
    }
    else if mm.msg_type == MSG_CORE_LIST || mm.msg_type == MSG_NEW_TRANSACTION || mm.msg_type == MSG_NEW_BLOCK || mm.msg_type == MSG_NEW_BLOCK_TO_ALL  || mm.msg_type == RSP_FULL_CHAIN || mm.msg_type == MSG_ENHANCED {
        res = vec![OK.to_string(), OK_WITH_PAYLOAD.to_string(), mm.msg_type.to_string(), mm.ip.to_string(), mm.port.to_string(), mm.payload.to_string()];
    }
    else {
        res = vec![OK.to_string(), OK_WITHOUT_PAYLOAD.to_string(), mm.msg_type.to_string(), mm.ip.to_string(), mm.port.to_string(), NONE.to_string()];
    }
    res

}

pub fn classify_msg(msg: &str) -> bool {
    let res = parse(msg);
    let msg_type = &res[2];
    let msg_type_num: usize = msg_type.parse().unwrap();
    if msg_type_num <= MSG_REMOVE_EDGE {
        return true;
    }
    return false;
}