use actix_web::web::{Data};
use actix_web::HttpMessage;
use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use hmac::Hmac;

use jwt::VerifyWithKey;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use sha2::Sha256;
use std::collections::BTreeMap;

use crate::user::models::User;
use crate::DB_NAME;

pub async fn user_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);

    match validate_user_token(credentials.token()) {
        Ok((res, reg_no)) => {
            if res == true {
                let client = req.app_data::<Data<Client>>().unwrap();
                let collection: Collection<User> = client.database(DB_NAME).collection("users");
                let user = collection
                    .find_one(doc! {"reg_no":&reg_no.unwrap()}, None)
                    .await;
                if let Ok(Some(user)) = user {
                    req.extensions_mut().insert::<User>(user);
                }
                Ok(req)
            } else {
                return Err((AuthenticationError::from(config).into(), req));
            }
        }
        Err(_) => Err((AuthenticationError::from(config).into(), req)),
    }
}
pub fn validate_user_token(token: &str) -> Result<(bool, Option<String>), Error> {
    let key: Hmac<Sha256> = hmac::Mac::new_from_slice(b"USER_SECRETE").unwrap();
    let result: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);

    match result {
        Ok(tree) => {
            let reg_no = tree.get("reg_no").unwrap().clone();
            return Ok((true, Some(reg_no)));
        }
        Err(_) => return Ok((false, None)),
    }
}

pub async fn admin_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
        let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);

    match validate_admin_token(credentials.token()) {
        Ok((res, username)) => {
            if res == true {
                let client = req.app_data::<Data<Client>>().unwrap();
                let collection: Collection<User> = client.database(DB_NAME).collection("admins");
                let user = collection
                    .find_one(doc! {"username":&username.unwrap()}, None)
                    .await;
                if let Ok(Some(user)) = user {
                    req.extensions_mut().insert::<User>(user);
                }
                Ok(req)
            } else {
                return Err((AuthenticationError::from(config).into(), req));
            }
        }
        Err(_) => Err((AuthenticationError::from(config).into(), req)),
    }

}
pub fn validate_admin_token(token: &str) -> Result<(bool, Option<String>), Error> {
    let key: Hmac<Sha256> = hmac::Mac::new_from_slice(b"ADMIN_SECRETE").unwrap();
    let result: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);

    match result {
        Ok(tree) => {
            let reg_no = tree.get("username").unwrap().clone();
            return Ok((true, Some(reg_no)));
        }
        Err(_) => return Ok((false, None)),
    }
}
