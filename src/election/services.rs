use std::f64::consts::E;

use actix_web::web;
use jwt::ToBase64;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{Acknowledgment, ReadConcern, TransactionOptions, WriteConcern},
    Client, Collection, Cursor,
};
use serde_json::json;

use crate::{
    blockchain::{BlockChain, Transaction},
    election::{self, types::ResultDTO},
    user::{models::User, utils::generate_keys},
    DB_NAME,
};

use super::{
    models::Election,
    types::{
        BeginElectionDTO, CreateElectionDTO, ElectionError, RegisterElectionDTO, StatusDTO, VoteDTO, ElectionDTO,
    },
};

const USER_COLLECTION: &str = "users";
const ELECTION_COLLECTION: &str = "election";

pub async fn create_election(
    client: web::Data<Client>,
    data: web::Json<CreateElectionDTO>,
) -> Result<String, ElectionError> {
    let user_collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);
    let election_collection: Collection<Election> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);

    // validate the candidate scopes
    if data.candidates.len() < 2 {
        return Err(ElectionError::InternalError {
            content: "Invalid number of candidates".into(),
        });
    }
    for candidate_pk in data.candidates.iter() {
        let candidate = user_collection
            .find_one(
                doc! {"$or":
                [{"public_key": &candidate_pk} ] },
                None,
            )
            .await;

        match candidate {
            Ok(Some(user)) => match data.scope.as_str() {
                "GENERAL" => {}
                "DEPARTMENT" => {
                    if user.department != data.scope_value {
                        return Err(ElectionError::ValidationError {
                            content: "Invalid scope candidate".into(),
                        });
                    }
                }
                "FACAULTY" => {
                    if user.facaulty != data.scope_value {
                        return Err(ElectionError::ValidationError {
                            content: "Invalid scope candidate".into(),
                        });
                    }
                }
                _ => {}
            },
            Ok(None) => {
                return Err(ElectionError::InternalError {
                    content: "Invalid candidate passed".into(),
                });
            }

            Err(e) => {
                return Err(ElectionError::InternalError {
                    content: e.to_string(),
                });
            }
        }
    }
    let result = election_collection
        .insert_one(
            Election {
                title: data.title.clone(),
                scope: data.scope.clone(),
                scope_value: data.scope_value.clone(),
                candidates: data.candidates.clone(),
                voters: data.candidates.clone(),
                status: "PENDING".into(),
                blockchain: None,
            },
            None,
        )
        .await;

    match result {
        Ok(r) => {
            let value = json!({
                "message": "Election successfully created",
                "election":  &r
            })
            .to_string();

            return Ok(value);
        }
        Err(e) => {
            return Err(ElectionError::InternalError {
                content: e.to_string(),
            });
        }
    }
}

pub async fn register_election(
    client: web::Data<Client>,
    data: web::Json<RegisterElectionDTO>,
) -> Result<String, ElectionError> {
    let user_collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);
    let election_collection: Collection<Election> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);

    let candidate = user_collection
        .find_one(
            doc! {"$or":
            [{"public_key": &data.pk} ] },
            None,
        )
        .await;

    let election_id = ObjectId::parse_str(&data.election_id).unwrap();
    let pk = &data.pk;

    let election = election_collection
        .find_one(
            doc! {
            "_id": election_id },
            None,
        )
        .await;
    if let Ok(Some(election)) = election {
        match candidate {
            Ok(Some(user)) => match election.scope.as_str() {
                "GENERAL" => {}
                "DEPARTMENT" => {
                    if user.department != election.scope_value {
                        return Err(ElectionError::ValidationError {
                            content: "Invalid scope candidate".into(),
                        });
                    }
                }
                "FACAULTY" => {
                    if user.facaulty != election.scope_value {
                        return Err(ElectionError::ValidationError {
                            content: "Invalid scope candidate".into(),
                        });
                    }
                }
                _ => {}
            },
            Ok(None) => {
                return Err(ElectionError::InternalError {
                    content: "Invalid candidate passed".into(),
                });
            }

            Err(e) => {
                return Err(ElectionError::InternalError {
                    content: e.to_string(),
                });
            } // validation complete
        }

        let result = election_collection
            .update_one(
                doc! {"_id":&election_id},
                doc! {
                    "$addToSet": {"voters": &pk}
                },
                None,
            )
            .await;

        println!("{:?}", result);

        if let Ok(result) = result {
            let value = json!({
                "message": "Election registeration succesfull",
                "election":  &result
            })
            .to_string();

            return Ok(value);
        } else {
            return Err(ElectionError::InternalError {
                content: "Error occurred updating database".into(),
            });
        }
    } else {
        return Err(ElectionError::InternalError {
            content: "Invalid election id".into(),
        });
    }
}

pub async fn begin_election(
    client: web::Data<Client>,
    data: web::Json<BeginElectionDTO>,
) -> Result<String, ElectionError> {
    let election_collection: Collection<Election> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);
    let election_id = ObjectId::parse_str(&data.election_id).unwrap();
    let election = election_collection
        .find_one(
            doc! {
            "_id": election_id },
            None,
        )
        .await;
    if let Ok(Some(election)) = election {
        if election.status == "PENDING".to_string() {
            let mut new_blockchain = BlockChain::new(Box::new(election.voters));
            let string_blockchain = new_blockchain.serialize();
            let result = election_collection
                .update_one(
                    doc! {"_id":&election_id},
                    doc! {
                        "$set":{
                            "status": "ONGOING",
                            "blockchain": &string_blockchain
                        }
                    },
                    None,
                )
                .await;

            println!("{:?}", result);

            let value = json!({
                "message": "Election succesfully started"
            })
            .to_string();

            return Ok(value);
        }
    }
    Err(ElectionError::InternalError {
        content: "Error occurred trying to update election".into(),
    })
}

pub async fn end_election(
    client: web::Data<Client>,
    data: web::Json<BeginElectionDTO>,
) -> Result<String, ElectionError> {
    let election_collection: Collection<Election> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);
    let election_id = ObjectId::parse_str(&data.election_id).unwrap();
    let election = election_collection
        .find_one(
            doc! {
            "_id": election_id },
            None,
        )
        .await;
    if let Ok(Some(election)) = election {
        if election.status == "ONGOING".to_string() {
            let _ = election_collection
                .update_one(
                    doc! {"_id":&election_id},
                    doc! {
                        "$set":{
                            "status": "ENDED",
                        }
                    },
                    None,
                )
                .await;

            let value = json!({
                "message": "Election ended started"
            })
            .to_string();

            return Ok(value);
        }
    }
    Err(ElectionError::InternalError {
        content: "Error occurred trying to update election".into(),
    })
}
pub async fn vote(
    client: web::Data<Client>,
    data: web::Json<VoteDTO>,
) -> Result<String, ElectionError> {
    let mut session = (client.start_session(None).await).unwrap();
    let election_collection: Collection<Election> = session
        .client()
        .database(DB_NAME)
        .collection(ELECTION_COLLECTION);
    let user_collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);

    let options = TransactionOptions::builder()
        .read_concern(ReadConcern::majority())
        .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
        .build();
    let _ = session.start_transaction(options).await;

    let keys = generate_keys(&data.password, &data.reg_no); // validate:: cannot vote for yourself
    let pk = keys.pk.to_base64().unwrap().to_string();
    let sk = keys.sk;

    let user = user_collection
        .find_one(
            doc! {
            "public_key": &pk },
            None,
        )
        .await;
    if let Ok(None) = user {
        return Err(ElectionError::InternalError {
            content: "Invalid Credentails".into(),
        });
    }

    let election_id = ObjectId::parse_str(&data.election_id).unwrap();
    let election = election_collection
        .find_one_with_session(
            doc! {
            "_id": election_id },
            None,
            &mut session,
        )
        .await;

    if let Ok(Some(election)) = election {
        if data.candidate_id != pk {
            let mut blockchain = BlockChain::from_string(election.blockchain.unwrap());
            let mut transaction = Transaction::new(pk, data.candidate_id.clone(), 1);
            let sign_result = &transaction.sign(sk);
            let result = blockchain.add_transaction(transaction.clone());
            println!("{:?} {:?}", transaction, result);

            if let Ok(_) = sign_result {
                if let Ok(_) = result {
                    let serialized = blockchain.serialize();
                    let result = election_collection
                        .update_one_with_session(
                            doc! {"_id":&election_id},
                            doc! {
                                "$set":{
                                    "status": "ONGOING",
                                    "blockchain": &serialized
                                }
                            },
                            None,
                            &mut session,
                        )
                        .await;
                    if let Ok(_) = result {
                        let value = json!({
                            "message": &"Voted successfully"
                        })
                        .to_string();
                        let _ = session.commit_transaction().await;

                        return Ok(value);
                    }
                }
            }
        }
    }
    let _ = session.abort_transaction().await;
    Err(ElectionError::InternalError {
        content: "Error occurred trying to update election".into(),
    })
}

pub async fn get_all(client: web::Data<Client>) -> Result<String, ElectionError> {
    let election_collection: Collection<ElectionDTO> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);

    let mut cursor = election_collection.find(None, None).await.unwrap();
    let mut elections: Vec<ElectionDTO> = vec![];

    while cursor.advance().await.unwrap() {
        if let Ok(election) = cursor.deserialize_current() {
            elections.push(election);
        }
    }
    let value = json!({
        "message": "Election succesfully retrieved",
        "elections": &elections
    })
    .to_string();

    return Ok(value);
}

pub async fn get_status(
    client: web::Data<Client>,
    data: web::Json<StatusDTO>,
) -> Result<String, ElectionError> {
    let election_collection: Collection<Election> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);
    let election_id = ObjectId::parse_str(&data.election_id).unwrap();
    let election = election_collection
        .find_one(
            doc! {
            "_id": election_id },
            None,
        )
        .await;

    if let Ok(Some(election)) = election {
        if election.status == "ONGOING" {
            let blockchain = BlockChain::from_string(election.blockchain.unwrap());
            let balance = blockchain.get_address_balance(&data.public_key);

            let status = if balance == 0 { "VOTED" } else { "NOT_VOTED" };
            let value = json!({ "status": &status }).to_string();
            return Ok(value);
        } else {
            let value = json!({ "status": &"NOT_VOTED" }).to_string();
            return Ok(value);
        }
    }

    Err(ElectionError::InternalError {
        content: "Error occurred trying to update election".into(),
    })
}

pub async fn get_candidates(
    client: web::Data<Client>,
    id: String,
) -> Result<String, ElectionError> {
    let election_collection: Collection<Election> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);
    let user_collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);

    let election_id = ObjectId::parse_str(&id).unwrap();
    let election = election_collection
        .find_one(
            doc! {
            "_id": election_id },
            None,
        )
        .await;
    let mut candidates: Vec<User> = vec![];

    if let Ok(Some(election)) = election {
        for c in election.candidates.iter() {
            let user = user_collection
                .find_one(
                    doc! {
                    "public_key": c },
                    None,
                )
                .await;
            if let Ok(Some(user)) = user {
                candidates.push(user);
            }
        }
        let value = json!({ "candidates": &candidates }).to_string();
        return Ok(value);
    }

    Err(ElectionError::InternalError {
        content: "Error occurred trying to update election".into(),
    })
}

pub async fn get_results(client: web::Data<Client>, id: String) -> Result<String, ElectionError> {

    let election_collection: Collection<Election> =
        client.database(DB_NAME).collection(ELECTION_COLLECTION);
    let user_collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);

    let election_id = ObjectId::parse_str(&id).unwrap();
    let election = election_collection
        .find_one(
            doc! {
            "_id": election_id },
            None,
        )
        .await;
    let mut results: Vec<ResultDTO> = vec![];

    if let Ok(Some(election)) = election {
        let bc = election.blockchain.clone().unwrap();
        let blockchain = BlockChain::from_string(bc);
        for c in election.candidates.iter() {
            let user = user_collection
                .find_one(
                    doc! {
                    "public_key": c },
                    None,
                )
                .await;
            if let Ok(Some(user)) = user {
                let pk = user.public_key.clone().unwrap();

                let result = ResultDTO {
                    name: user.name,
                    reg_no: user.reg_no,
                    public_key: pk.clone(),
                    votes: blockchain.get_address_balance(&pk) as i32,
                };
                results.push(result);
            }
        }
        let value = json!({ "candidates": &results }).to_string();
        return Ok(value);
    }
   Err(ElectionError::InternalError {
        content: "Error occurred trying to update election".into(),
    })
}
