use std::{sync::Arc, time::Duration};

use axum::{routing::get, Router};
use http::request::Request;
use hyper::Body;
use libsql_client::Client;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};

mod logging;
use logging::init_logger;

mod api;
use api::{create_users, get_users};
use tracing::{Level, Span};

mod json;
mod trace;

#[shuttle_runtime::main(tracing_layer = init_logger)]
async fn axum(
    #[shuttle_turso::Turso(
        addr = "libsql://advanced-lightspeed-kazy.turso.io",
        token = "{secrets.TURSO_DB_TOKEN}"
    )]
    client: Client,
) -> shuttle_axum::ShuttleAxum {
    let client = Arc::new(client);

    client
        .execute("create table if not exists example_users ( uid text primary key, email text );")
        .await
        .unwrap();

    let service = ServiceBuilder::new().layer(TraceLayer::new_for_http());
    let router = Router::new()
        .route("/", get(get_users).post(create_users))
        .with_state(client)
        .layer(service);

    Ok(router.into())
}
