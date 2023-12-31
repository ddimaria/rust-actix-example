//! Spin up a HTTPServer

use crate::utils;
use crate::cache::add_cache;
use crate::config::CONFIG;
use crate::database::add_pool;
use crate::routes::routes;
use crate::state::new_state;
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::time::Duration;
use actix_web::http::header;
use actix_web::{middleware::{self}, App, HttpServer, cookie::Key};
use listenfd::ListenFd;

pub async fn server() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Create the application state
    // String is used here, but it can be anything
    // Invoke in hanlders using data: AppState<'_, String>
    let data = new_state::<String>();
    let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_owned());
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .configure(add_cache)
            .wrap(
                Cors::default()
                    .allowed_origin(&CONFIG.server)
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600))
            // Identity management
            .wrap(IdentityMiddleware::default())
            // Session
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(utils::SECRET_KEY.as_bytes()),
                )
                .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                .cookie_name("auth-example".to_owned())
                .cookie_secure(false)
                .cookie_domain(Some(domain.clone()))
                .cookie_path("/".to_owned())
                .build(),
            ) // enable logger
            .wrap(middleware::Logger::default())
            .configure(add_pool)
            
            .configure(routes)
    }).workers(2);

    server = if let Some(l) = listenfd.take_tcp_listener(0)? {
        server.listen(l)?
    } else {
        server.bind(&CONFIG.server)?
    };

    log::info!("starting HTTP server at http://{}", &CONFIG.server);

    server.run().await
}
