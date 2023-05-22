use crate::{
    election::{
        services::{
            begin_election, create_election, end_election, get_all, get_candidates, get_results,
            get_status, register_election, vote,
        },
        types::{
            BeginElectionDTO, CreateElectionDTO, ElectionError, RegisterElectionDTO, StatusDTO,
            VoteDTO,
        },
    },
    middleware::auth::{admin_validator, user_validator},
    user::{
        models::User,
        services::{admin_login, login, sign_up},
        types::{AdminLoginDTO, UserError, UserLoginDTO},
    },
};
use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    get, post, web, App, Error,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use mongodb::Client;
#[post("/create")]
async fn handle_create(
    client: web::Data<Client>,
    data: web::Json<CreateElectionDTO>,
) -> Result<String, ElectionError> {
    create_election(client, data).await
}

#[post("/begin")]
async fn handle_begin(
    client: web::Data<Client>,
    data: web::Json<BeginElectionDTO>,
) -> Result<String, ElectionError> {
    begin_election(client, data).await
}

#[post("/end")]
async fn handle_end(
    client: web::Data<Client>,
    data: web::Json<BeginElectionDTO>,
) -> Result<String, ElectionError> {
    end_election(client, data).await
}

#[post("/now")]
async fn handle_vote(
    client: web::Data<Client>,
    data: web::Json<VoteDTO>,
) -> Result<String, ElectionError> {
    vote(client, data).await
}
//get candidates
//get voting state in get status
//end elections

#[post("/get_status")]
async fn handle_get_status(
    client: web::Data<Client>,
    data: web::Json<StatusDTO>,
) -> Result<String, ElectionError> {
    get_status(client, data).await
}

#[post("/register")]
async fn handle_register(
    client: web::Data<Client>,
    data: web::Json<RegisterElectionDTO>,
) -> Result<String, ElectionError> {
    register_election(client, data).await
}

#[get("/all")]
async fn handle_get_all(client: web::Data<Client>) -> Result<String, ElectionError> {
    get_all(client).await
}

#[get("/results/{id}")]
async fn handle_get_results(
    client: web::Data<Client>,
    path: web::Path<String>,
) -> Result<String, ElectionError> {
    let id = path.into_inner();
    get_results(client, id).await
}

#[get("/candidates/{id}")]
async fn handle_get_candidates(
    client: web::Data<Client>,
    path: web::Path<String>,
) -> Result<String, ElectionError> {
    let id = path.into_inner();
    get_candidates(client, id).await
}

pub fn register_controllers<
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
>(
    app: App<T>,
) -> App<T> {
    app.service(
        web::scope("/election")
            .service(handle_create)
            .service(handle_end)
            .service(handle_begin)
            .service(handle_get_results)
            .service(handle_get_all)
            .wrap(HttpAuthentication::bearer(admin_validator)),
    )
    .service(
        web::scope("/vote")
            .service(handle_get_status)
            .service(handle_vote)
            .service(handle_register)
            .service(handle_get_candidates)
            .wrap(HttpAuthentication::bearer(user_validator)),
    )
}
