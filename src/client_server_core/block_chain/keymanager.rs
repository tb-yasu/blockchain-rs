/*  
    keymanager.rs
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

//! It manages the private_key and public_key, and it computes a digital signature using those keys. 

extern crate minisign;
use minisign::{KeyPair, PublicKeyBox, SecretKeyBox, SignatureBox};
use std::io::Cursor;

extern crate rand;
use rand::seq::SliceRandom;

const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub struct KeyManager {
    pub private_key_str: String,
    pub public_key_str: String,
    password: String
}

impl KeyManager {
    pub fn create(random_num: usize) -> KeyManager {
        //! create private_key and public_key with a random number

        let random_pass = gen_ascii_chars(random_num);
        let KeyPair {pk, sk} = KeyPair::generate_encrypted_keypair(Some(random_pass.clone())).unwrap();

        let private_key_str = sk.to_box(None).unwrap().to_string();
        let public_key_str = pk.to_box().unwrap().to_string();
        println!("private key: {:#?}", private_key_str);
        println!("public key: {:#?}", public_key_str);

        KeyManager {    
            private_key_str: private_key_str,
            public_key_str: public_key_str,
            password: random_pass.clone()
        }
    }

    pub fn my_address(&self) -> String {
        //! return the address as a public_key

        self.public_key_str.clone()
    }

    pub fn compute_digital_signature(&self, message: &str) -> String {
        let sk_box = SecretKeyBox::from_string(&self.private_key_str).unwrap();
        let sk = sk_box
        .into_secret_key(Some(self.password.clone()))
        .unwrap();

        let msg_reader = Cursor::new(message);
        let signature_box = minisign::sign(None, &sk, msg_reader, None, None).unwrap();

        signature_box.into_string()
    }

    pub fn verify_signature(&self, message: &str, signature_box_str: &str, sender_public_key_box_str: &str) -> bool {
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

    pub fn export_key_pair(&mut self, key_data: &str, pass_phrase: &str) -> (String, String) {
        return (self.private_key_str.clone(), self.public_key_str.clone());
    }

    pub fn import_key_pair(&mut self, private_key_str: &str, public_key_str: &str) {
        self.private_key_str = private_key_str.to_string();
        self.public_key_str = public_key_str.to_string();
    }

    pub fn clone(&self) -> KeyManager {
        KeyManager { 
            private_key_str: self.private_key_str.clone(),
            public_key_str: self.public_key_str.clone(), 
            password: self.password.clone()
        }
    }


}

fn gen_ascii_chars(size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR.as_bytes()
            .choose_multiple(&mut rng, size)
            .cloned()
            .collect()
    ).unwrap()
}

pub fn run() {
    let km = KeyManager::create(40);

    let my_address = km.my_address();
    println!("my_address: {}", my_address);

    let msg = "The first message";
    let signature = km.compute_digital_signature(&msg);

    let flag = km.verify_signature(msg, &signature, &km.public_key_str);
    println!("verify suffcess {}", flag);

    let msg2 = "The second message";
    let flag2 = km.verify_signature(msg2, &signature, &km.public_key_str);
    println!("verify suffcess {}", flag2);

}