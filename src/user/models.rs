use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub name: String,
    pub reg_no: String,
    pub password: String,
    pub email: String,
    pub department: String,
    pub facaulty: String,
    pub public_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Admin {
    pub username: String,
    pub password: String,
}
