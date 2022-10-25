use anyhow;
use axum::{
    extract::{Extension, Form, Path},
    http::StatusCode,
    response::{Html, Redirect},
};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio;
use tower_http::services::ServeDir;

mod repo;
mod template;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::net::SocketAddr;

    // Initialize database
    let addr: SocketAddr = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() == 2 {
            args[1].parse()?
        } else {
            println!("usage: eisenhower-todo ADDRESS");
            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            println!("using default address({addr})");
            addr
        }
    };

    let app = newapp();

    println!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

fn newapp() -> axum::Router {
    use axum::routing::{get, get_service, post};
    use repo::Repo;

    let static_files =
        get_service(ServeDir::new("./static")).handle_error(|err: std::io::Error| async move {
            (StatusCode::NOT_FOUND, format!("Not Found: {}", err))
        });
    let mut repo = Repo::new();
    repo.add("Test item", "Body of test item", true, false)
        .expect("Failed to insert test item");
    let repomux = Arc::new(Mutex::new(repo));

    axum::Router::new()
        .route("/", get(get_index))
        .route("/item/:id", get(get_item))
        .route("/item/:id/edit", get(get_edit_item).post(post_edit_item))
        .route("/item/:id/delete", post(post_delete_item))
        .layer(Extension(repomux))
        .nest("/static", static_files)
}

type AppError = (StatusCode, String);

// Handlers

async fn get_index(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
) -> Result<Html<String>, AppError> {
    let repo = lock_repo(&repomux)?;
    let items = repo.all()?;
    let viewmodel = template::Index::from_items(&items);
    let body = viewmodel.to_string();
    Ok(Html(body))
}

async fn get_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Html<String>, AppError> {
    let repo = lock_repo(&repomux)?;
    let item = repo
        .get(item_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, String::from("No such item")))?;
    let viewmodel: template::Item = item.into();
    let body = viewmodel.to_string();
    Ok(Html(body))
}

async fn get_edit_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Html<String>, AppError> {
    let repo = lock_repo(&repomux)?;
    let item = repo
        .get(item_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, String::from("No such item")))?;
    let viewmodel: template::EditItem = item.into();
    let body = viewmodel.to_string();
    Ok(Html(body))
}

async fn post_edit_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
    Form(edits): Form<EditParams>,
) -> Result<Redirect, AppError> {
    let mut repo = lock_repo(&repomux)?;
    let mut item = repo
        .get(item_id)
        .ok_or((StatusCode::NOT_FOUND, "No such item".to_owned()))?
        .clone();
    item.apply(&edits);
    repo.upsert(&item)?;
    Ok(Redirect::to(&format!("/item/{}", item.id)))
}

async fn post_delete_item(
    Extension(repomux): Extension<Arc<Mutex<repo::Repo>>>,
    Path(item_id): Path<u32>,
) -> Result<Redirect, AppError> {
    let mut repo = lock_repo(&repomux)?;
    repo.delete(&item_id)?;
    Ok(Redirect::to("/"))
}

// get_new_item
// post_new_item

#[derive(serde::Deserialize)]
struct EditParams {
    pub title: String,
    pub body: String,
    pub important: Option<String>,
    pub urgent: Option<String>,
}

impl repo::Item {
    fn apply(&mut self, edits: &EditParams) {
        self.title.clear();
        self.title.insert_str(0, &edits.title);
        self.body.clear();
        self.body.insert_str(0, &edits.body);
        self.important = edits.important.is_some();
        self.urgent = edits.urgent.is_some();
    }
}

// Helpers
fn lock_repo<'a>(repomux: &'a Arc<Mutex<repo::Repo>>) -> Result<MutexGuard<repo::Repo>, AppError> {
    repomux.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Couldn't lock the item repo"),
        )
    })
}
