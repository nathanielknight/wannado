use anyhow;
use axum::{extract::Extension, http::StatusCode, response::Html};
use std::sync::{Arc, Mutex};
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

    let static_files =
        get_service(ServeDir::new("./static")).handle_error(|err: std::io::Error| async move {
            (StatusCode::NOT_FOUND, format!("Not Found: {}", err))
        });
    let repo = Arc::new(Mutex::new(repo::Repo::new()));

    axum::Router::new()
        .route("/", get(get_index))
        .layer(Extension(repo))
        .nest("/static", static_files)
}

type AppError = (StatusCode, String);

// Handlers

async fn get_index(repo: Extension<Arc<Mutex<repo::Repo>>>) -> Result<Html<String>, AppError> {
    let repo = repo.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Couldn't lock the item repo"),
        )
    })?;
    let items = repo.all()?;
    let viewmodel = template::Index::from_items(&items);
    let body = viewmodel.to_string();
    Ok(Html(body))
}

