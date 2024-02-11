use std::{collections::HashMap, sync::Arc};

use askama::Template;
use axum::{debug_handler, extract::{Query, State}, response::{Html, IntoResponse}, routing::get, Form, Router};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower_http::{services::ServeDir, trace::TraceLayer};
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

#[derive(Clone)]
struct BaseUrl(Arc<str>);

#[derive(Clone)]
struct ChatiplexState {
    chats: Chats,
    base_url: BaseUrl,
}

pub fn chatiplex(url_prefix: impl Into<Arc<str>>, assets_dir: &str) -> Router {
    let url_prefix = url_prefix.into();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .ok();

    let chats = Chats::new();

    let state = ChatiplexState {
        chats: chats.clone(),
        base_url: BaseUrl(url_prefix.clone()),
    };

    // print working directory for debugging purposes
    debug!("Current working directory: {:?}", std::env::current_dir().unwrap());

    Router::new()
        .route("/", get(index))
        .route("/chats", get(get_chat).post(post_chat))
        .nest_service("/assets", ServeDir::new(assets_dir))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn index(State(state): State<ChatiplexState>) -> impl IntoResponse {
    Html(
        templates::IndexTemplate { base_url: state.base_url.0.as_ref() }.render().unwrap()
    )
}

#[derive(Deserialize, Debug)]
struct GetChatQuery {
    id: String,
}

async fn get_chat(
    State(state): State<ChatiplexState>,
    Query(input): Query<GetChatQuery>,
) -> impl IntoResponse
{
    match state.chats.0.read().await.get(&input.id) {
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
    State(state): State<ChatiplexState>,
    Form(input): Form<PostChatForm>,
) -> impl IntoResponse
{
    debug!("Form data: {:?}", input);

    state.chats.0
        .write().await
        .entry(input.id.clone())
        .or_insert(Chat { id: input.id.clone(), messages: Vec::new() })
        .messages.push(input.message);

    get_chat(State(state), Query(GetChatQuery { id: input.id })).await
}
