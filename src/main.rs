use anyhow;
use axum::response::Html;
use tokio;
use tower_http::services::ServeDir;

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
    use axum::http::StatusCode;
    use axum::routing::{get, get_service};

    let static_files =
        get_service(ServeDir::new("./static")).handle_error(|err: std::io::Error| async move {
            (StatusCode::NOT_FOUND, format!("Not Found: {}", err))
        });

    axum::Router::new()
        .route("/", get(get_index))
        .nest("/static", static_files)
}

// Handlers

async fn get_index() -> Html<String> {
    Html(String::from("Hello, app!"))
}
