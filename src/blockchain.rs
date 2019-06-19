use serde::{Serialize, Deserialize};
use crypto::digest::Digest;
use std::time::SystemTime;
use crypto::sha2::Sha256;
use std::sync::Mutex;
use std::sync::Arc;
use url::Url;
use std::str;

type Link = Arc<Mutex<Vec<Transaction>>>;

#[derive(Serialize, Deserialize)]
pub struct Block {
    index: i32,
    timestamp: u64,
    transactions: Link,
    pub proof: i32,
    previous_hash: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: String
}

pub struct BlockChain {
    current_transactions: Link,
    pub chain: Vec<Block>,
    pub nodes: Vec<String>
}


impl BlockChain {

    pub fn new() -> Self {
        let mut blockchain = BlockChain {
            current_transactions: Arc::new(Mutex::new(Vec::new())),
            chain: Vec::new(),
            nodes: Vec::new()
        };

        blockchain.new_block(1, Some("100".to_string()));
        blockchain
    }

    pub fn new_block(&mut self, proof: i32, previous_hash: Option<String>) -> Option<&Block> {
        let time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n,
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let block = Block {
            index: self.chain.len() as i32 + 1,
            timestamp: time.as_secs(),
            transactions: self.current_transactions.clone(),
            proof: proof,
            previous_hash: previous_hash
        };

        self.current_transactions = Arc::new(Mutex::new(Vec::new()));
        self.chain.push(block);
        self.chain.last()
    }

    pub fn new_transaction(&mut self, sender: String, recipient: &String, amount: String) -> i32 {
        let transaction = Transaction {
            sender: sender,
            recipient: recipient.to_string(),
            amount: amount
        };
        self.current_transactions.lock().unwrap().push(transaction);
        return match self.chain.last() {
            Some(block) => {
                block.index + 1
            },
            None => {
                1
            }
        }
    }

    pub fn last_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn register_node(&mut self, address: String) {
        let parsed_url = Url::parse(&address);
        match parsed_url {
            Ok(url) => {
                match url.host() {
                    Some(host) => {
                        self.nodes.push(host.to_string());
                    },
                    None => {
                        return
                    }
                }
            },
            Err(_) => {
                return
            }
        }
    }

    pub fn resolve_conflicts(&mut self) -> bool {
        let mut max_length = self.chain.len();
        let nodes = &self.nodes;
        let mut chain = Vec::new();
        for node in nodes {
            let dt = match BlockChain::requisition(&format!("http://{}/chain", node)) {
                Ok(vec) => vec,
                Err(_) => Vec::new()
            };

            if dt.len() > max_length && self.valid_chain(&dt) {
                max_length = dt.len();
                chain = dt;
            }
        }
        if max_length > self.chain.len() {
            self.chain = chain;
            true
        } else {
            false
        }
    }

    fn requisition(url: &str) -> Result<Vec<Block>, Box<std::error::Error>> {
        let resp: Vec<Block> = reqwest::get(url)?
            .json()?;
        Ok(resp)
    }

    fn valid_chain(&self, chain: &Vec<Block>) -> bool {

        for block in chain {
            if block.previous_hash != Some(Self::hash(&chain[0])) {
                return false
            }

            if !Self::valid_proof(&chain[0].proof.to_string(), block.proof.to_string()) {
                return false
            }
        }
        return true
    }

    pub fn proof_of_work(last_proof: i32) -> i32 {
        let mut proof = 0;
        while Self::valid_proof(&last_proof.to_string(), proof.to_string()) == false {
            proof = proof + 1;
        }
        proof
    }


    fn valid_proof(last_proof: &String, proof: String) -> bool {
        let mut hasher = Sha256::new();
        let guess = format!("{}{}", last_proof, proof);
        hasher.input_str(&guess);
        let guess_hash = hasher.result_str();
        &guess_hash[..6] == "000000"
    }

    pub fn hash(block: &Block) -> String {
        let mut hasher = Sha256::new();
        let block_str: String = json!(block).to_string();
        hasher.input_str(&block_str);
        hasher.result_str()
    }

}