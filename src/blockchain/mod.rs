use std::{collections::HashMap, fmt::format, io::Read, mem, time::SystemTime};

pub mod controllers;
pub mod models;
pub mod services;
pub mod types;
pub mod utils;

use actix_web::{
    web::{block, Json},
    Error,
};
use chrono::Utc;
use ed25519_compact::{SecretKey, Signature};
use jwt::ToBase64;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha256VarCore};

use crate::{
    common::utils::{from_base64, to_base64},
    user::utils::pk_from_string,
};

use self::types::BlockchainError;

const VOID_ADDRESS: &'static str = "0000000000000000";
const VOID_HASH: &'static str = "0000000000000000";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub timestamp: i64,
    signature: Option<String>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: i64) -> Self {
        Transaction {
            from,
            to,
            amount,
            timestamp: Utc::now().timestamp(),
            signature: None,
        }
    }

    pub fn sign(&mut self, sk: SecretKey) -> Result<bool, BlockchainError> {
        let pk = sk.public_key().to_base64().unwrap().to_string();
        if pk != self.from {
            println!("error my nigga");
            return Err(BlockchainError::TransactionError {
                content: "Invalid Transaction:: Cannot sign transactions for other wallets".into(),
            });
        }
        let hash = self.calculate_hash();
        println!("{}", hash);
        let sig = sk.sign(hash, None);
        self.signature = Some(to_base64(&sig.to_vec()));
        Ok(true)
    }

    pub fn calculate_hash(&self) -> String {
        let hash_input = format!("{}{}{}{}", self.from, self.to, self.amount, self.timestamp);
        let mut hash = Sha256::new();
        hash.update(hash_input);
        format!("{:X}", hash.finalize())
    }

    pub fn is_valid(&self) -> Result<bool, BlockchainError> {
        if self.from == VOID_ADDRESS.to_string() {
            return Ok(true);
        }
        if self.signature == None {
            return Err(BlockchainError::TransactionError {
                content: "Invalid Transaction:: Empty signature".into(),
            });
        }

        let pk = pk_from_string(&self.from);
        let sig_array = from_base64(self.signature.as_ref().unwrap()).unwrap();

        let sig = Signature::from_slice(sig_array.as_slice()).unwrap();

        if let Ok(_) = pk.verify(self.calculate_hash(), &sig) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Block {
    pub previous: Option<String>,
    pub timestamp: i64,
    pub transactions: Box<Vec<Transaction>>,
    pub hash: Option<String>,
}
impl Block {
    pub fn new(
        previous: Option<String>,
        timestamp: i64,
        transactions: Box<Vec<Transaction>>,
    ) -> Self {
        let mut block = Block {
            previous,
            timestamp,
            transactions,
            hash: None,
        };
        let hash = block.calculate_hash();
        block.hash = Some(hash);
        block
    }

    pub fn calculate_hash(&self) -> String {
        let hash_input = format!(
            "{}{}{}",
            self.previous.as_ref().unwrap_or(&"".to_string()),
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap()
        );
        let mut hash = Sha256::new();
        hash.update(hash_input);
        format!("{:X}", hash.finalize())
    }

    pub fn is_valid(&self) -> bool {
        for transaction in self.transactions.iter() {
            if let Ok(false) = transaction.is_valid() {
                return false;
            }
        }

        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BlockChain {
    chain: Vec<Block>,
    transaction_buffer: Vec<Transaction>,
}

impl BlockChain {
    pub fn new(registered_addresses: Box<Vec<String>>) -> Self {
        let mut block_chain = BlockChain {
            chain: vec![],
            transaction_buffer: vec![],
        };
        let genesis_block = block_chain.create_genesis_block(registered_addresses);
        block_chain.chain.push(genesis_block);
        block_chain
    }

    pub fn create_genesis_block(&self, registered_addresses: Box<Vec<String>>) -> Block {
        let mut genesis_transactions: Box<Vec<Transaction>> = Box::new(vec![]);
        for address in registered_addresses.iter() {
            genesis_transactions.push(Transaction::new(VOID_ADDRESS.into(), address.clone(), 1))
        }
        Block::new(Some(VOID_HASH.into()), 0, genesis_transactions)
    }
    pub fn get_latest_block(&self) -> &Block {
        &self.chain[self.chain.len() - 1]
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), BlockchainError> {
        if Some(&transaction.from) == Some(&VOID_ADDRESS.into())
            || Some(&transaction.to) == Some(&VOID_ADDRESS.into())
        {
            return Err(BlockchainError::TransactionError {
                content: "Invalid Transaction:: Invalid to or from address".into(),
            });
        }
        if !transaction.is_valid().unwrap() {
            return Err(BlockchainError::TransactionError {
                content: "Invalid Transaction:: Transaction not valid".into(),
            });
        }
        if transaction.amount <= 0 {
            return Err(BlockchainError::TransactionError {
                content: "Invalid Transaction:: Transaction amount should be greater than zero"
                    .into(),
            });
        }

        let wallet_balance = self.get_address_balance(&transaction.from);
        if wallet_balance < transaction.amount {
            return Err(BlockchainError::TransactionError {
                content: "Invalid Transaction:: Insufficient balance".into(),
            });
        }

        if self.transaction_buffer.len() > 0 {
            let pending_wallet_transactions: Vec<&Transaction> = self
                .transaction_buffer
                .iter()
                .filter(|t| t.from == transaction.from)
                .collect();

            let total_pending_amount = pending_wallet_transactions
                .iter()
                .map(|t| t.amount)
                .reduce(|p, c| p + c)
                .unwrap();
            if total_pending_amount > wallet_balance {
                return Err(BlockchainError::TransactionError {
                    content: "Invalid Transaction:: Pending transaction amount exceeded".into(),
                });
            }
        }

        if self.transaction_buffer.len() < 10 {
            self.transaction_buffer.push(transaction);
        } else {
            self.create_block_from_buffer();
            self.transaction_buffer.clear();
            self.transaction_buffer.push(transaction);
        }
        Ok(())
    }

    pub fn get_address_balance(&self, address: &String) -> i64 {
        let mut balance = 0;
        for block in &self.chain {
            for trans in block.transactions.as_ref().iter() {
                if trans.from == *address {
                    balance -= trans.amount
                }
                if trans.to == *address {
                    balance += trans.amount
                }
            }
        }
        balance
    }

    pub fn create_block_from_buffer(&mut self) {
        let latest_block = &self.get_latest_block();
        let trx = &self.transaction_buffer;
        if trx.len() > 0 {
            let block = Block::new(
                Some(latest_block.hash.clone().unwrap()),
                Utc::now().timestamp(),
                Box::new(trx.clone()),
            );
            self.chain.push(block)
        }
    }

    pub fn is_valid(&self, registered_addresses: Box<Vec<String>>) -> bool {
        let real_genesis = self.create_genesis_block(registered_addresses);
        let current_genesis = self.chain.get(0);

        if serde_json::to_string(&real_genesis).unwrap()
            != serde_json::to_string(&current_genesis.unwrap()).unwrap()
        {
            println!("got here");
            return false;
        }
        for i in 1..self.chain.len() {
            let current_block = self.chain.get(i).unwrap();
            let prev_block = self.chain.get(i - 1).unwrap();

            if prev_block.hash != current_block.hash {
                return false;
            }
            if !current_block.is_valid() {
                return false;
            }

            if current_block.hash != Some(current_block.calculate_hash()) {
                return false;
            }
        }
        true
    }

    pub fn serialize(&mut self) -> String {
        if self.transaction_buffer.len() > 0 {
            self.create_block_from_buffer();
            self.transaction_buffer.clear();
        }
        serde_json::to_string(&self.chain).unwrap()
    }

    pub fn deserialize(&mut self, json: String) {
        let chain: Result<Vec<Block>, _> = serde_json::from_str(json.as_str());
        if let Ok(chain) = chain {
            self.chain = chain;
        }
    }

    pub fn from_string(s: String) -> Self {
        let mut bc = BlockChain::new(Box::new(vec![]));
        bc.deserialize(s);
        bc
    }
}
