use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Error)]
pub enum ElectionError {
    #[display(fmt = "{}", content)]
    ValidationError { content: String },

    #[display(fmt = "{}", content)]
    InternalError { content: String },
}

impl error::ResponseError for ElectionError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            ElectionError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            ElectionError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CreateElectionDTO {
    pub title: String,
    pub scope: String,
    pub scope_value: String,
    pub candidates: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RegisterElectionDTO {
    pub pk: String,
    pub election_id: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BeginElectionDTO {
    pub election_id: String,

}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct VoteDTO {
    pub election_id: String,
    pub password: String,
    pub reg_no: String,
    pub candidate_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StatusDTO {
    pub election_id: String,
    pub public_key: String,
}


#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResultDTO {
    pub name: String,
    pub reg_no: String,
    pub public_key: String,
    pub votes: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ElectionDTO{
    pub _id: Option<ObjectId>,
    pub title: String,
    pub scope: String,
    pub scope_value: String,
    pub candidates: Vec<String>,
    pub voters: Vec<String>,
    pub status: String,
    pub blockchain: Option<String>,
}