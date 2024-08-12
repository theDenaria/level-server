use std::time::Duration;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgPoolOptions, prelude::FromRow, query_file, query_file_as, query_file_scalar, PgPool,
};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    let app = Router::new()
        .route("/get-objects", get(get_objects)) // will be called Denaria server to fetch Level's physics objects.
        .route("/get-object", get(get_object)) // will be called Denaria server to fetch Level's physics objects.
        .route("/set-objects", post(set_objects)) // will be called from Unity Level development scene manually
        .with_state(pool);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct GetObjectParams {
    id: i32,
}

async fn get_object(
    State(pool): State<PgPool>,
    Query(params): Query<GetObjectParams>,
) -> Result<Json<LevelObject>, (StatusCode, String)> {
    let result = query_file_as!(LevelObject, "queries/get_object_by_id.sql", params.id)
        .fetch_one(&pool)
        .await;
    //.map_err(internal_error);

    tracing::info!("Request came with id: {}", params.id);

    match result {
        Ok(object) => Ok(Json(object)),
        Err(e) => Err(internal_error(e)),
    }
}

async fn get_objects(
    State(pool): State<PgPool>,
) -> Result<Json<GetObjectsResonse>, (StatusCode, String)> {
    let result = query_file_as!(LevelObject, "queries/get_objects.sql")
        .fetch_all(&pool)
        .await;
    //.map_err(internal_error);

    match result {
        Ok(objects) => Ok(Json(GetObjectsResonse { objects })),
        Err(e) => Err(internal_error(e)),
    }
}

async fn set_objects(
    State(pool): State<PgPool>,
    Json(req): Json<SetLevelObjectRequest>,
) -> Result<Json<SetObjectsResonse>, (StatusCode, String)> {
    let SetLevelObjectRequest {
        object_type,
        position,
        rotation,
        scale,
        collider,
    } = req;

    let set_result = query_file!(
        "queries/set_object.sql",
        object_type,
        position,
        rotation,
        scale,
        collider,
    )
    .execute(&pool)
    .await;
    tracing::trace!(
        "type: {}, color: {}, position: {:?}, scale: {:?}, set_result: {:?}",
        object_type,
        position,
        scale,
        rotation,
        set_result
    );

    let count_result = query_file_scalar!("queries/get_row_count.sql")
        .fetch_one(&pool)
        .await
        .map_err(internal_error);

    tracing::trace!("count_result: {:?}", count_result);

    match count_result {
        Ok(count_op) => match count_op {
            Some(count) => {
                return Ok(Json(SetObjectsResonse {
                    count,
                    success: true,
                }));
            }
            None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "No count".to_string())),
        },
        Err((_, em)) => Err(internal_error_from_string(em)),
    }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn internal_error_from_string(err_string: String) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err_string)
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct LevelObject {
    id: i32,
    object_type: String,
    position: String,
    rotation: String,
    scale: String,
    collider: String,
}

#[derive(Serialize, Deserialize)]
struct GetObjectsResonse {
    objects: Vec<LevelObject>,
}

#[derive(Serialize, Deserialize)]
struct GetObjectByIdResonse {
    object: LevelObject,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetLevelObjectRequest {
    object_type: String,
    position: String,
    rotation: String,
    scale: String,
    collider: String,
}

// #[derive(Serialize, Deserialize)]
// struct SetObjectsRequest {
//     objects: Vec<SetLevelObject>,
// }

#[derive(Serialize, Deserialize)]
struct SetObjectsResonse {
    count: i64,
    success: bool,
}

// async fn create_user(
//     State(pool): State<deadpool_diesel::postgres::Pool>,
//     Json(new_user): Json<NewUser>,
// ) -> Result<Json<User>, (StatusCode, String)> {
//     let conn = pool.get().await.map_err(internal_error)?;
//     let res = conn
//         .interact(|conn| {
//             diesel::insert_into(users::table)
//                 .values(new_user)
//                 .returning(User::as_returning())
//                 .get_result(conn)
//         })
//         .await
//         .map_err(internal_error)?
//         .map_err(internal_error)?;
//     Ok(Json(res))
// }
