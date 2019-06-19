#![feature(proc_macro_hygiene, decl_macro, const_fn)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket_contrib;
extern crate reqwest;

mod blockchain;
use std::thread;
use std::sync::Mutex;
use rocket_contrib::json::JsonValue;
use rocket_contrib::json::Json;
use blockchain::{BlockChain, Transaction};
use uuid::Uuid;


lazy_static! {
    static ref NODEID: String = Uuid::new_v4().to_simple().to_string();
    static ref BLOCKCHAIN: Mutex<BlockChain> = Mutex::new(BlockChain::new());
}

#[get("/mine")]
fn mine() -> JsonValue {
    thread::spawn(||{
        let mut blockchain = BLOCKCHAIN.lock().unwrap();
        blockchain.new_transaction("0".to_string(), &*NODEID, "1".to_string());
    });
    
    let mut blockchain = BLOCKCHAIN.lock().unwrap();
    let last_block = match blockchain.last_block() {
        Some(block) => block,
        None =>  return json!({"error": true})
    };
    let last_proof = last_block.proof;
    let proof = BlockChain::proof_of_work(last_proof);

    let previous_hash = BlockChain::hash(last_block);
    let block = blockchain.new_block(proof, Some(previous_hash));
    json!(block)
}

#[post("/transactions/new", data="<transaction>")]
fn new_transaction(transaction: Json<Transaction>) -> JsonValue {
    let mut blockchain = BLOCKCHAIN.lock().unwrap();
    let transaction = transaction.into_inner();

    let index = blockchain.new_transaction(transaction.sender, &transaction.recipient, transaction.amount);
    json!(index)
}

#[get("/chain")]
fn full_chain() -> JsonValue {
    let blockchain = BLOCKCHAIN.lock().unwrap();
    json!({
        "chain": blockchain.chain,
        "lenght": blockchain.chain.len()
    })
}

#[post("/nodes/register", data="<nodes>")]
fn register_node(nodes: Json<Vec<String>>) -> JsonValue {
    let mut blockchain = BLOCKCHAIN.lock().unwrap();
    for node in nodes.into_inner() {
        blockchain.register_node(node);
    }
    json!({
        "message": "New nodes has been added",
        "total_nodes": blockchain.nodes.len()
    })
}
#[get("/nodes/resolve")]
fn consensus() -> JsonValue {
    let mut blockchain = BLOCKCHAIN.lock().unwrap();
    let replaced = blockchain.resolve_conflicts();
    json!({
        "new_chain": replaced,
        "chain": blockchain.chain
    })
}


fn main() {
    rocket::ignite()
        .mount("/", routes![mine, new_transaction, full_chain, register_node])
        .launch();
}

