# blockchain-rs

Blockchain-rs is a rust implementation of blockchain.  It utilizes several edge nodes and core nodes as a base. An edge node is a client that can send and receive a remittance information. A core node is a server that receives a remittance information from edge nodes and register the information into a blockchain. The proof-of-work algorithm is used as a consensus algorithm. 

Blockchain-rs has a simple example. A sender as an edge node sends 30 coins with 5 fee to a receiver as another edge node. A core node receive the remittance information from the sender and register it to the block chain. The blockchain is shared with another core node. 
