mod api;
mod model;
mod repository;

use std::io::Error;

use api::task::get_task;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use aws_config::BehaviorVersion;
use repository::ddb::DDBRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    HttpServer::new(move || {
        let logger = Logger::default();
        let ddb_repo = DDBRepository::init("tasks".to_string(), config);
        let ddb_data = Data::new(ddb_repo);
        App::new().wrap(logger).app_data(ddb_data).service(get_task)
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}
