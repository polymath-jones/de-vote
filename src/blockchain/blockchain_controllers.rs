use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    get, App, Error, HttpResponse, Responder,
};

#[get("/blockchain")]
async fn test() -> impl Responder {
    HttpResponse::Ok().body("Testing blockchain")
}

pub fn register<T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>>(
    app: App<T>,
) -> App<T> {
    app
    .service(test)
}
