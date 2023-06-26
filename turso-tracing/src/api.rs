use std::sync::Arc;
use tracing::{info, instrument};

use axum::{extract, extract::State, response::IntoResponse, Json};
use libsql_client::{args, Client, Row, Statement, Value};
use serde::{Deserialize, Serialize};

fn row_string_field(r: &Row, index: usize) -> String {
    match r.values.get(index).unwrap() {
        Value::Text { value } => value.clone(),
        _ => unreachable!(),
    }
}

#[axum::debug_handler]
#[instrument(skip(client))]
pub async fn get_users(State(client): State<Arc<Client>>) -> Json<Vec<User>> {
    let rows = client.execute("select * from example_users").await.unwrap();
    let users: Vec<_> = rows
        .rows
        .iter()
        .map(|r| User {
            uid: row_string_field(r, 0),
            email: row_string_field(r, 1),
        })
        .collect();
    Json(users)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    uid: String,
    email: String,
}

#[axum::debug_handler]
#[instrument(skip(client))]
pub async fn create_users(
    State(client): State<Arc<Client>>,
    extract::Json(user): extract::Json<User>,
) -> impl IntoResponse {
    info!("creating new user");
    client
        .execute(Statement::with_args(
            "insert into example_users values (?, ?)",
            args!(user.uid, user.email),
        ))
        .await
        .unwrap();

    Json(serde_json::json!({ "ok": true }))
}
