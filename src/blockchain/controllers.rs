use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    get, web, App, Error, HttpResponse, Responder,
};
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::{blockchain::BlockChain, middleware::auth::{user_validator, admin_validator}, user::utils::generate_keys};

#[get("")]
async fn test() -> impl Responder {
    let pk = String::from("WzgzLDI0LDEzNiwxMDQsMTg3LDcwLDE2NywxNzIsMjQ2LDE3MSwyMzcsMTY1LDgyLDI0NywxLDIwNiw0OSw4MSw4Myw2LDE3OSwxODcsODYsMjAzLDEwLDkxLDYzLDI1MSwyNTEsMjE3LDE1NywyMjdd");

    // let mut  t = Transaction::new(pk,"000xx".into(),3);
    // t.calculate_hash();
    // let fakekey = generate_keys(&"password".into(),&"182830994".into());
    // let _ = t.sign(fakekey.sk);

    let mut bc = BlockChain::new(Box::new(vec![pk.clone()]));
    let s = bc.serialize();

    let mut bc2 = BlockChain::new(Box::new(vec![]));
    bc2.deserialize(s);

    let balance = bc.get_address_balance(&pk);
    println!("{:?} {:?}", bc, balance);
    HttpResponse::Ok().body("Testing blockchain")
}

pub fn register_controllers<
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
>(
    app: App<T>,
) -> App<T> {
    app.service(
        web::scope("/blockchain")
            .service(test)
            .wrap(HttpAuthentication::bearer(admin_validator)),
    )
}
