use axum::{extract::Extension, http::StatusCode};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

mod handlers;
mod repo;
mod script;
mod template;

// ------------------------------------------------------
// Helpers
pub(crate) type AppError = (StatusCode, String);

#[tokio::main]
async fn main() {
    use std::net::SocketAddr;

    // Initialize database
    let addr: SocketAddr = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() == 2 {
            args[1]
                .parse()
                .expect("Expected a socket address. eg:\n127.0.0.1:3000")
        } else {
            println!("usage: eisenhower-todo ADDRESS");
            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            println!("using default address({addr})");
            addr
        }
    };

    let app = newapp();

    script::start_recurring_script();
    println!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}

fn newapp() -> axum::Router {
    use axum::routing::{get, get_service, post};
    use repo::Repo;

    let static_files =
        get_service(ServeDir::new("./static")).handle_error(|err: std::io::Error| async move {
            (StatusCode::NOT_FOUND, format!("Not Found: {}", err))
        });
    let cxn = rusqlite::Connection::open("./items.sqlite3").expect("Couldn't open database");
    let mut repo = Repo::new(cxn);
    repo.init().expect("Database initialisation failed");
    repo.add("Test item", "Body of test item", true, false)
        .expect("Failed to insert test item");
    let repomux = Arc::new(Mutex::new(repo));

    axum::Router::new()
        .route("/", get(handlers::get_index))
        .route(
            "/item/new",
            get(handlers::get_new_item).post(handlers::post_new_item),
        )
        .route("/item/:id", get(handlers::get_item))
        .route(
            "/item/:id/edit",
            get(handlers::get_edit_item).post(handlers::post_edit_item),
        )
        .route("/item/:id/delete", post(handlers::post_delete_item))
        .layer(Extension(repomux))
        .nest("/static", static_files)
}
