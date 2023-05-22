mod blockchain;
mod common;
mod user;
mod middleware;
mod election;

use actix_cors::Cors;
use actix_service::ServiceFactory;
use actix_web::{web, App, HttpServer, dev::{ServiceRequest, ServiceResponse,}, Error};
use common::utils::create_generic_index;
use election::models::Election;
use mongodb::Client;
use user::models::User;


const DB_NAME: &str = "devote";


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "".into());
    let client = Client::with_uri_str(uri)
        .await
        .expect("failed to connect to database client");

    create_generic_index::<User>(&client, "reg_no".into(), "users".into()).await;
    create_generic_index::<User>(&client, "public_key".into(), "users".into()).await;
    // create_generic_index::<Election>(&client, "".into(), "elections".into()).await;
    
    HttpServer::new(move || {
        let cors = Cors::permissive();
        let mut app = App::new().wrap(cors);//.wrap(cors);
        app = election::controllers::register_controllers(app);
        app = user::controllers::register_controllers(app);
        app = blockchain::controllers::register_controllers(app);
        app = app.app_data(web::Data::new(client.clone()));
        app
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
