use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};


#[derive(Debug, Display, Error)]
pub enum BlockchainError {
    #[display(fmt = "{}", content)]
    ValidationError { content: String },

    #[display(fmt = "{}", content)]
    InternalError { content: String },

    #[display(fmt = "{}", content)]
    TransactionError { content: String },
}

impl error::ResponseError for BlockchainError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            BlockchainError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            BlockchainError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            BlockchainError::TransactionError { .. } => StatusCode::NOT_ACCEPTABLE
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DTO {
    pub r: String,
    pub p: String,
}
