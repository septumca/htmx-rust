use anyhow::anyhow;
use axum::{
    self,
    routing::{get, delete},
    response::{Response, IntoResponse, Html},
    http::StatusCode,
    Form, Router, extract::{State, Path},
};
use serde::Deserialize;
use sha256::digest;
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
    id: i64,
    title: String,
    creator: String
}

#[derive(Deserialize)]
struct User {
    id: i64,
    username: String
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
        .route("/login", get(login).post(do_login))
        .route("/story/create", get(story_create))
        .route("/story", get(story_list).post(create_story))
        .route("/story/:id", delete(delete_story))
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

async fn root() -> Result<Html<String>, AppError> {
    let template = RootTemplate { };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

async fn login() -> Result<Html<String>, AppError> {
    let template = LoginTemplate { };
    Ok(Html(template.render()?))
}

#[derive(Deserialize, Debug)]
struct LoginInput {
    username: String,
    password: String,
}

async fn do_login(
    State(state): State<Arc<AppState>>,
    Form(input): Form<LoginInput>
) -> Result<Response, AppError> {
    let mut conn = state.pool.acquire().await?;
    let user = sqlx::query!(r#"
            SELECT user.password, user.salt
            FROM user
            WHERE user.username = ?1
        "#,
        input.username)
        .fetch_one(&mut *conn)
        .await?;
    let db_password = digest(user.password + &user.salt);
    let input_password = digest(input.password + &user.salt);

    if db_password != input_password {
        return Err(AppError(anyhow!("Login failed!")));
    }

    Ok(([("HX-Redirect", "/"), ("Set-Cookie", "session-id:12345")]).into_response())
}

#[derive(Template)]
#[template(path = "story-list.html")]
struct StoryListTemplate {
    story_list: Vec<Story>,
}

async fn story_list(
    State(state): State<Arc<AppState>>
) -> Result<Html<String>, AppError> {
    let story_list = sqlx::query_as!(Story, r#"
        SELECT story.id, story.title, user.username as creator
        FROM story
        JOIN user on user.id = story.creator
    "#)
    .fetch_all(&state.pool)
    .await?;

    let template = StoryListTemplate { story_list };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "story-create.html")]
struct StoryCreateTemplate {
    user_list: Vec<User>,
}

async fn story_create(
    State(state): State<Arc<AppState>>
) -> Result<Html<String>, AppError> {
    let user_list = sqlx::query_as!(User, r#"
        SELECT id, username
        FROM user
    "#)
    .fetch_all(&state.pool)
    .await?;

    let template = StoryCreateTemplate { user_list };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "story-list-element.html")]
struct StoryElementTemplate {
    story: Story,
}

#[derive(Deserialize, Debug)]
struct NewStoryInput {
    creator: i64,
    title: String,
}

async fn create_story(
    State(state): State<Arc<AppState>>,
    Form(input): Form<NewStoryInput>
) -> Result<Html<String>, AppError> {
    let mut conn = state.pool.acquire().await?;

    let creator = sqlx::query!(r#"
        SELECT username FROM user WHERE id = ?1
        "#,
        input.creator,
    )
    .fetch_one(&mut *conn)
    .await?;

    let id = sqlx::query!(r#"
        INSERT INTO Story (title, creator)
        VALUES(?1, ?2)
        "#,
        input.title,
        input.creator,
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    let template = StoryElementTemplate {
        story: Story {
            id,
            title: input.title,
            creator: creator.username
        }
    };
    Ok(Html(template.render()?))
}

async fn delete_story(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>
) -> Result<(), AppError> {
    let mut conn = state.pool.acquire().await?;

    sqlx::query!(r#"
        DELETE FROM Story WHERE id = ?1
        "#,
        id,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}
