
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Election{
    pub title: String,
    pub scope: String,
    pub scope_value: String,
    pub candidates: Vec<String>,
    pub voters: Vec<String>,
    pub status: String,
    pub blockchain: Option<String>,
}