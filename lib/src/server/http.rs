// http - Creates and register HTTP server
// Sasaki, Naoki <nsasaki@sal.co.jp> October 16, 2022
//

use crate::context::session_manager::SessionManager;
use crate::server::routes;
use crate::settings::Settings;
use axum::http::{header, Method};
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn create_server<S: SessionManager>(
    session_mgr: Arc<tokio::sync::Mutex<S>>,
) -> Result<(axum::serve::Serve<axum::Router, axum::Router>, SocketAddr), anyhow::Error> {
    let app = routes::register::<S>(session_mgr).layer(
        tower_http::cors::CorsLayer::new()
            .allow_headers(vec![
                header::ACCEPT,
                header::ACCEPT_LANGUAGE,
                header::AUTHORIZATION,
                header::CONTENT_LANGUAGE,
                header::CONTENT_TYPE,
            ])
            .allow_methods(vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_origin(tower_http::cors::Any)
            .allow_credentials(false),
    );

    let sock_addr = SocketAddr::from(([0, 0, 0, 0], Settings::global().server.port));
    let listener = tokio::net::TcpListener::bind(sock_addr).await?;

    Ok((axum::serve(listener, app), sock_addr))
}
