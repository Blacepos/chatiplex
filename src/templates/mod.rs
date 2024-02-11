
use askama::Template;

#[derive(Template)]
#[template(path = "index.askama.html")]
pub struct IndexTemplate<'a> {
    pub base_url: &'a str,
}

#[derive(Template)]
#[template(path = "chats.askama.html")]
pub struct ChatsTemplate<'a> {
    pub chats: Vec<ChatTemplate<'a>>,
}

#[derive(Template)]
#[template(path = "chat.askama.html")]
pub struct ChatTemplate<'a> {
    pub id: &'a str,
    pub messages: Vec<String>,
}
