use actix_web::{middleware, web, App, HttpServer};
use actix_files::Files;

use dotenv::dotenv;
use std::sync::{Mutex, Arc};
use actix_web::dev::ResourceDef;

mod db;
mod api;
mod utils;

use structopt::StructOpt;



#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Config {
    /// data folder
    #[structopt(short, long, default_value = "./")]
    folder: String,

    /// Listen ip
    #[structopt(long, default_value = "127.0.0.1", env = "PANDA_API_HOST")]
    host: String,

    /// Listen port
    #[structopt(long, default_value = "9000", env = "PANDA_API_PORT")]
    port: usize,
}



#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    pretty_env_logger::init();
    let conf = Config::from_args();

    let db = db::Database::load();

    let web_db = web::Data::new(Mutex::new(db));

    utils::watch_api_docs_change(web_db.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web_db.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2").header("Access-Control-Allow-Origin", "*"))

            .service(web::resource("/index").route(web::get().to(api::server_info)))
            .service(web::resource("/__api_docs/").route(web::get().to(api::get_api_doc_basic)))
            .service(web::resource("/__api_docs/api_data/").route(web::get().to(api::get_api_doc_data)))
            .service(web::resource("/__api_docs/_data/").route(web::get().to(api::get_api_doc_schema_data)))
            .service(Files::new("/js", "theme/js"))
            .service(Files::new("/css", "theme/css"))
            .service(
                web::resource("/*")
                    .route(web::get().to(api::action_handle))
                    .route(web::post().to(api::action_handle))
                    .route(web::put().to(api::action_handle))
                    .route(web::delete().to(api::action_handle))
            )
    })
        .bind(format!("{}:{}", conf.host, conf.port))?
        .run()
        .await
}
