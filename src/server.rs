extern crate actix_web;

use actix_web::{App, HttpResponse, HttpRequest, HttpServer, middleware, web, Error, error};
use std::sync::{Arc, Mutex};
use crate::config::Config;
use crate::storage::{Storage, LogItem};
use crate::database::DBManager;
use crate::crypto_functional::{from_base64, sign_and_encrypt, verify_and_decrypt, DecriptionError, to_base64};


async fn get_handler(info: web::Path<String>, storage: web::Data<Arc<Mutex<Storage>>>, db_manager: web::Data<DBManager>) -> Result<HttpResponse, Error> {
    let data: String = match storage.lock().unwrap().get_all(&info) {
        Some(d) => d.iter().map(|x| { format!("{}</\n/>{}/\n/>{}", x.ip, x.time, x.data) }).collect::<Vec<String>>().join("\n--==--\n"),
        None => return Err(error::ErrorNotFound("No data was found".to_string()))
    };
    let user = match db_manager.get_user(&info) {
        Ok(d) => d,
        Err(_err) => return Err(error::ErrorNotFound("No key was found".to_string()))
    };

    let key = match from_base64(&user.key) {
        Ok(d) => d,
        Err(_) => return Err(error::ErrorInternalServerError("Error on reading the key".to_string()))
    };

    let to_send = sign_and_encrypt(&data.as_bytes().to_vec(), &key);

    Ok(HttpResponse::Ok().body(to_send))
}


async fn push_handler(req: HttpRequest, info: web::Path<String>, form: web::Form<String>, storage: web::Data<Arc<Mutex<Storage>>>, db_manager: web::Data<DBManager>) -> Result<HttpResponse, Error> {
    let user = match db_manager.get_user(&info) {
        Ok(d) => d,
        Err(_err) => return Err(error::ErrorNotFound("No key was found".to_string()))
    };

    let key = match from_base64(&user.key) {
        Ok(d) => d,
        Err(_) => return Err(error::ErrorInternalServerError("Error on reading the key".to_string()))
    };

    let data = match verify_and_decrypt(&form.0, &key) {
        Ok(d) => d,
        Err(err) => {
            return match err {
                DecriptionError::VerificationFailure => Err(error::ErrorBadRequest("Corrupted data: please resend".to_string())),
                DecriptionError::Unknown => Err(error::ErrorInternalServerError("Unknown error: resend, please".to_string())),
                DecriptionError::WrongFormat => Err(error::ErrorNotAcceptable("Wrong format".to_string())),
                DecriptionError::Base64Failure => Err(error::ErrorNotAcceptable("Wrong format: Base64 error".to_string()))
            };
        }
    };

    let ip = match req.connection_info().remote() {
        Some(d) => d.to_string(),
        None => "NOREMOTE".to_string()
    };
    match storage.lock().unwrap().push(&user.name, LogItem::new(&ip, &to_base64(&data))) {
        Ok(_) => Ok(HttpResponse::Ok().body("OK")),
        Err(err) => Err(error::ErrorInternalServerError(format!("Error on writing to the storage: {:?}", err)))
    }
}

#[actix_rt::main]
pub async fn run_server(a_config: Arc<Mutex<Config>>) {
    let config: Config = { a_config.lock().unwrap().clone() };
    let ds = Arc::new(Mutex::new(Storage::new(config.logsize)));
    let db = DBManager::open(&config.db_config).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(ds.clone()).data(db.clone())
            .service(web::resource("/push/{username}").route(web::post().to(push_handler)))
            .service(web::resource("/get/{username}").to(get_handler))
    }
    )
        .bind(config.bind_address)
        .unwrap()
        .run().await.unwrap();
}