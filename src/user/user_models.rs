use std::collections::HashMap;

use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub name: String,
    pub reg_no: String,
    pub password: String,
    pub email: String,
    pub department: String,
    pub faculty: String,
    pub age: Option<i32>,
}



#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ValidationError {
    pub message: String,
    pub error_code: u16,
    pub fields: Option< HashMap<String,String>>

}