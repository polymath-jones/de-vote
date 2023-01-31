
use std::string;

use crate::{user::user_models::{User, ValidationError}, DB_NAME};
use actix_web::{
    dev::{ServiceFactory,ServiceRequest}, post, web, App, Error, HttpResponse, Responder,error, http::{header::ContentType, StatusCode}
};
use serde_json::json;
use derive_more::{Display, Error};
use mongodb::{Client, Collection, bson::doc};
const COLLECTION_NAME: &str = "users";

#[derive(Debug,Display,Error)]
enum UserError{
    #[display(fmt = "{}",content)]
    ValidationError{content:String},
}

impl error::ResponseError for UserError{
    fn error_response(&self)-> HttpResponse {
        HttpResponse::build(self.status_code())
        .insert_header(ContentType::json())
        .body(self.to_string())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
       match *self {
        UserError::ValidationError {..} => StatusCode::BAD_REQUEST
       } 
    }
}

#[post("/user/signup")]
async fn signup(client:web::Data<Client>, data: web::Json<User>) -> Result<&'static str, UserError> {

    let collection:Collection<User> = client.database(DB_NAME).collection(COLLECTION_NAME);
    let user = collection.find_one(doc!{"reg_no": &data.reg_no,"email":&data.email }, None).await;
    
    if let Ok(user) = user {
        match user {
           Some(..)=> {

            let error = ValidationError{message:"".into(),error_code: StatusCode::BAD_REQUEST.as_u16(),fields:None};
            return Err( UserError::ValidationError { content: serde_json::to_value(error).unwrap().to_string()  });
           },
           None=> {}, 
        }
    }
    Ok("Sign up Successfull")
}

pub fn register<T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>>(
    app: App<T>,
) -> App<T> {
    app.service(signup)
}

/*
 * a user can sign up
 * a user can sign in
 * a user can recover password
 * a user can be verified
 * a user can alpply for verification
 */