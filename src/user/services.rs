use super::{
    models::{Admin, User},
    types::{AdminLoginDTO, UserError, UserLoginDTO},
};
use crate::{user::utils::generate_keys, DB_NAME};
use actix_web::web;
use bcrypt;
use hmac::Hmac;
use jwt::{SignWithKey, ToBase64};
use mongodb::{bson::doc, Client, Collection};
use serde_json::json;
use sha2::Sha256;
use std::collections::BTreeMap;
use str;

const USER_COLLECTION: &str = "users";
const ADMIN_COLLECTION: &str = "admins";

pub async fn sign_up(
    client: web::Data<Client>,
    data: web::Json<User>,
) -> Result<String, UserError> {
    let collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);
    let user = collection
        .find_one(
            doc! {"$or": [{"reg_no": &data.reg_no},{"email": &data.email } ] },
            None,
        )
        .await;

    match user {
        Ok(Some(_)) => {
            let value = json!({
                "message": "user with registration number or email already exists"
            });
            return Err(UserError::ValidationError {
                content: value.to_string(),
            });
        }
        Ok(None) => {
            let hashed_password = bcrypt::hash(&data.password, 10).unwrap();
            let mut user = data.into_inner();
            let unhashed_password = user.password;
            user.password = hashed_password;

            let pair = generate_keys(&unhashed_password, &user.reg_no);
            let pk = pair.pk.to_base64().unwrap().to_string();
            user.public_key = Some(pk);

            let result = collection.insert_one(&user, None).await;
            println!("{:?} ", result);
            match result {
                Ok(..) => {
                    let key: Hmac<Sha256> = hmac::Mac::new_from_slice(b"USER_SECRETE").unwrap();
                    let mut claims: BTreeMap<String, String> = BTreeMap::new();
                    claims.insert("reg_no".into(), user.reg_no.clone());
                    let token = claims.sign_with_key(&key);

                    let value = json!({
                        "message": "signup successfull",
                        "token": token.unwrap()
                    })
                    .to_string();

                    return Ok(value);
                }
                Err(e) => {
                    return Err(UserError::InternalError {
                        content: e.to_string(),
                    });
                }
            }
        }
        Err(_) => {
            return Err(UserError::InternalError {
                content: "Error occcured fetching user".into(),
            })
        }
    }
}

pub async fn login(
    client: web::Data<Client>,
    data: web::Json<UserLoginDTO>,
) -> Result<String, UserError> {
    let collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);
    let user = collection
        .find_one(doc! {"reg_no":&data.reg_no }, None)
        .await;

    match user {
        Ok(Some(user)) => {
            if let Ok(true) = bcrypt::verify(&data.password, &user.password) {
                let key: Hmac<Sha256> = hmac::Mac::new_from_slice(b"USER_SECRETE").unwrap();
                let mut claims: BTreeMap<String, String> = BTreeMap::new();
                claims.insert("reg_no".into(), data.reg_no.clone());
                let token = claims.sign_with_key(&key);

                let value = json!({
                    "message": "login successfull",
                    "token": token.unwrap(),
                    "me": &user
                })
                .to_string();

                return Ok(value);
            }
        }

        Ok(None) => {}
        Err(e) => {
            return Err(UserError::InternalError {
                
                content: e.to_string(),
            });
        }
    }
    Err(UserError::InternalError { content: "".into() })
}
pub async fn admin_login(
    client: web::Data<Client>,
    data: web::Json<AdminLoginDTO>,
) -> Result<String, UserError> {
    let collection: Collection<Admin> = client.database(DB_NAME).collection(ADMIN_COLLECTION);
    let user = collection
        .find_one(doc! {"username":&data.username }, None)
        .await;

    match user {
        Ok(Some(user)) => {
            if let Ok(true) = bcrypt::verify(&data.password, &user.password) {
                let key: Hmac<Sha256> = hmac::Mac::new_from_slice(b"ADMIN_SECRETE").unwrap();
                let mut claims: BTreeMap<String, String> = BTreeMap::new();
                claims.insert("username".into(), data.username.clone());

                let token = claims.sign_with_key(&key);
                let value = json!({
                    "message": "Admin login successfull",
                    "token": token.unwrap()
                })
                .to_string();
                return Ok(value);
            }
        }

        Ok(None) => {
        
        }
        Err(e) => {
            return Err(UserError::InternalError {
                content: e.to_string(),
            });
        }
    }
    Err(UserError::InternalError { content: "".into() })
}


pub async fn get_user(client: web::Data<Client>,reg_no: String)  -> Result<String, UserError>{

    let collection: Collection<User> = client.database(DB_NAME).collection(USER_COLLECTION);
    let user = collection
        .find_one(doc! {"reg_no":&reg_no }, None)
        .await;

    match user {
        Ok(Some(user)) => {
                let value = json!(user)
                .to_string();
                return Ok(value);   
        }
        Ok(None) => {}
        Err(e) => {
            return Err(UserError::InternalError {
                content: e.to_string(),
            });
        }
    }
    Err(UserError::InternalError { content: "".into() })
        

}