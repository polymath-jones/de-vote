use std::string::ParseError;

use jwt::{ToBase64, FromBase64};
use mongodb::{Client, options::IndexOptions, IndexModel, bson::doc};
use crate::DB_NAME;


pub async fn create_generic_index<T>(client: &Client, field: String, collection: String){
    
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
    .keys(doc!{field:1})
    .options(options)
    .build();

    client.database(DB_NAME)
    .collection::<T>(&collection)
    .create_index(model,None)
    .await
    .expect("creatring an index should succeed");
}

pub fn to_base64(vec:&Vec<u8>) -> String {
    ToBase64::to_base64(&vec).unwrap().to_string()
}

pub fn from_base64(s:&String) -> Result<Vec<u8>,jwt::Error>{
   let pk_dec: Result<Vec<u8>, jwt::Error> = FromBase64::from_base64(s);
   pk_dec
}