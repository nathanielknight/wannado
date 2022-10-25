use anyhow;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::Html,
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
    use axum::routing::{get, get_service};
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
        .route("/item/:id/edit", get(get_edit_item))
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


// post_edit_item
// post_delete_item
// get_new_item
// post_new_item

// Helpers
fn lock_repo<'a>(repomux: &'a Arc<Mutex<repo::Repo>>) -> Result<MutexGuard<repo::Repo>, AppError> {
    repomux.lock().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Couldn't lock the item repo"),
        )
    })
}
