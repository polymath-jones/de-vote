
use actix_web::{App, HttpServer, web};
use common::utils::create_generic_index;
use mongodb::{Client};
use user::user_models::User;
mod user;
mod blockchain;
mod common;

const DB_NAME: &str = "devote";

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "".into());
    let client = Client::with_uri_str(uri).await.expect("failed to connect to database client");
    create_generic_index::<User>(&client, "reg_no".into(), "users".into()).await;

    HttpServer::new( move || {
        let mut app = App::new();
        app = blockchain::blockchain_controllers::register(app);
        app = user::user_controllers::register(app);
        app.app_data(web::Data::new(client.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}



