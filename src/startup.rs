use actix_web::dev::Server;
use actix_web::web::{get, post, Data};
use actix_web::{App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // wrap the pool using web::Data, which boils down to an Arc smart pointer
    let db_pool = Data::new(db_pool);

    // Capture `connection` from the surrounding environment using `move`
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", get().to(health_check))
            .route("/subscriptions", post().to(subscribe))
            // Get a pointer copy and attach it to the application state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
