use axum::{
    self,
    routing::{get, post},
    http::StatusCode,
    response::{Response, IntoResponse, Html},
    Form, Router, extract::State,
};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use askama::Template;
use sqlx::sqlite::SqlitePool;
use dotenv_rs::dotenv;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        ).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Deserialize)]
struct Story {
    title: String,
    creator: String
}

struct AppState {
    pool: SqlitePool
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>{
    // initialize tracing
    tracing_subscriber::fmt::init();
    dotenv().ok();
    let db_url = dotenv_rs::var("DATABASE_URL")?;
    tracing::info!("database url: {}", db_url);
    let pool = SqlitePool::connect(&db_url).await?;
    let shared_state = Arc::new(AppState { pool });

    let app = Router::new()
        .route("/", get(root))
        .route("/story", get(story_list))
        .with_state(shared_state)
        ;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}


#[derive(Template)]
#[template(path = "index.html")]
struct RootTemplate {}

async fn root<'a>(
) -> Result<Html<String>, AppError> {
    let template = RootTemplate { };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "story-list.html")]
struct StoryListTemplate {
    story_list: Vec<Story>
}

async fn story_list<'a>(
) -> Result<Html<String>, AppError> {
    let template = StoryListTemplate {
        story_list: vec![
            Story {
                title: "Test".into(),
                creator: "Jason".into()
            }
        ]
    };
    Ok(Html(template.render()?))
}