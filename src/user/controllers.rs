use crate::{
    middleware::auth::admin_validator,
    user::{
        models::User,
        services::{admin_login, get_user, login, sign_up},
        types::{AdminLoginDTO, UserError, UserLoginDTO},
    },
};
use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    get, post, web, App, Error,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use mongodb::Client;

#[get("/get_user/{reg_no}")]
async fn handle_get_user(
    client: web::Data<Client>,
    path: web::Path<String>,
) -> Result<String, UserError> {
    let reg_no = path.into_inner();
    get_user(client, reg_no).await
}

#[post("/admin_login")]
async fn handle_admin_login(
    client: web::Data<Client>,
    data: web::Json<AdminLoginDTO>,
) -> Result<String, UserError> {
    admin_login(client, data).await
}

#[post("/signup")]
async fn handle_sign_up(
    client: web::Data<Client>,
    data: web::Json<User>,
) -> Result<String, UserError> {
    sign_up(client, data).await
}

#[post("/login")]
async fn handle_login(
    client: web::Data<Client>,
    data: web::Json<UserLoginDTO>,
) -> Result<String, UserError> {

    // println!("{:?}",data);
    // Ok("sfsfsf".into())
    login(client, data).await
}

pub fn register_controllers<
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
>(
    app: App<T>,
) -> App<T> {
    app.service(
        web::scope("/user")
            .service(handle_login)
            .service(handle_sign_up)
            .service(handle_admin_login),
    )
    .service(
        web::scope("/admin")
            .service(handle_get_user)
            .wrap(HttpAuthentication::bearer(admin_validator)),
    )
}
