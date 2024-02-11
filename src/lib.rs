use std::{collections::HashMap, sync::Arc};

use askama::Template;
use axum::{debug_handler, extract::{Query, State}, response::{Html, IntoResponse}, routing::get, Form, Router};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::debug;

mod templates;

struct Chat {
    id: String,
    messages: Vec<String>,
}

#[derive(Clone)]
pub struct Chats(Arc<RwLock<HashMap<String, Chat>>>);

impl Chats {
    fn new() -> Self {
        Chats(Arc::new(RwLock::new(HashMap::new())))
    }
}

pub fn chatiplex() -> Router {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .ok();

    let chats = Chats::new();

    Router::new()
        .route("/", get(index))
        .route("/chats", get(get_chat).post(post_chat))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .with_state(chats)
        .layer(TraceLayer::new_for_http())
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../assets/index.html"))
}

#[derive(Deserialize, Debug)]
struct GetChatQuery {
    id: String,
}

async fn get_chat(
    State(chats): State<Chats>,
    Query(input): Query<GetChatQuery>,
) -> impl IntoResponse
{
    match chats.0.read().await.get(&input.id) {
        Some(chat) => Html(
            templates::ChatTemplate {
                id: &chat.id,
                messages: chat.messages.clone()
            }
            .render().unwrap()
        ),
        None => Html(
            "<p>This chat is empty! Be the first to send a message!</p>".to_string()
        ),
    }
}

#[derive(Deserialize, Debug)]
struct PostChatForm {
    id: String,
    message: String
}

#[debug_handler]
async fn post_chat(
    State(chats): State<Chats>,
    Form(input): Form<PostChatForm>,
) -> impl IntoResponse
{
    debug!("Form data: {:?}", input);

    chats.0
        .write().await
        .entry(input.id.clone())
        .or_insert(Chat { id: input.id.clone(), messages: Vec::new() })
        .messages.push(input.message);

    get_chat(State(chats), Query(GetChatQuery { id: input.id })).await
}
