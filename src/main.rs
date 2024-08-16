use std::time::Duration;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use queries::{
    create_table_sql, delete_all_sql, get_first_id_sql, get_object_by_id_sql, get_objects_sql,
    get_row_count_sql, set_object_sql,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, prelude::FromRow, query, query_as, query_scalar, PgPool};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod queries;

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
        .route("/prepare", get(prepare_table))
        .route("/get-objects", get(get_objects))
        .route("/get-object", get(get_object))
        .route("/get-first", get(get_first_id))
        .route("/set-object", post(set_object)) // will be called from Unity Level development scene manually
        .with_state(pool);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct GetObjectParams {
    version: String,
    id: i32,
}

async fn get_object(
    State(pool): State<PgPool>,
    Query(params): Query<GetObjectParams>,
) -> Result<Json<LevelObject>, (StatusCode, String)> {
    let query_string = get_object_by_id_sql(params.version);
    let result = query_as::<_, LevelObject>(query_string.as_str())
        .bind(params.id)
        .fetch_one(&pool)
        .await;

    match result {
        Ok(object) => Ok(Json(object)),
        Err(e) => Err(internal_error(e)),
    }
}

#[derive(Debug, Deserialize)]
struct GetFirstIdParams {
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetFirstIdResponse {
    id: i32,
}

async fn get_first_id(
    State(pool): State<PgPool>,
    Query(params): Query<GetFirstIdParams>,
) -> Result<Json<GetFirstIdResponse>, (StatusCode, String)> {
    let query_string = get_first_id_sql(params.version);
    let result: Result<Option<i32>, (StatusCode, String)> = query_scalar(query_string.as_str())
        .fetch_one(&pool)
        .await
        .map_err(internal_error);
    match result {
        Ok(id_result) => match id_result {
            Some(id) => {
                return Ok(Json(GetFirstIdResponse { id }));
            }
            None => Err(internal_error_from_string("No id found".to_string())),
        },
        Err(e) => Err(e),
    }
}

#[derive(Debug, Deserialize)]
struct GetAllObjectsParams {
    version: String,
}
async fn get_objects(
    State(pool): State<PgPool>,
    Query(params): Query<GetAllObjectsParams>,
) -> Result<Json<GetObjectsResponse>, (StatusCode, String)> {
    let query_string = get_objects_sql(params.version);
    let result = query_as::<_, LevelObject>(query_string.as_str())
        .fetch_all(&pool)
        .await;

    match result {
        Ok(objects) => Ok(Json(GetObjectsResponse { objects })),
        Err(e) => Err(internal_error(e)),
    }
}

async fn set_object(
    State(pool): State<PgPool>,
    Json(req): Json<SetLevelObjectRequest>,
) -> Result<Json<SetObjectsResonse>, (StatusCode, String)> {
    let SetLevelObjectRequest {
        version,
        object_type,
        position,
        rotation,
        scale,
        collider,
    } = req;

    let query_string = set_object_sql(version.clone());

    let _set_result = query(query_string.as_str())
        .bind(&object_type)
        .bind(&position)
        .bind(&rotation)
        .bind(&scale)
        .bind(&collider)
        .execute(&pool)
        .await;

    let query_string = get_row_count_sql(version);

    let count_result = query_scalar(query_string.as_str())
        .fetch_one(&pool)
        .await
        .map_err(internal_error);

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

#[derive(Debug, Deserialize)]
struct PrepareTableParams {
    version: String,
}
async fn prepare_table(
    State(pool): State<PgPool>,
    Query(params): Query<PrepareTableParams>,
) -> Result<Json<SetObjectsResonse>, (StatusCode, String)> {
    let query_string = create_table_sql(params.version.clone());
    let set_result = query(query_string.as_str()).execute(&pool).await;

    let query_string = delete_all_sql(params.version.clone());
    let set_result = query(query_string.as_str()).execute(&pool).await;

    let query_string = get_row_count_sql(params.version);

    let count_result = query_scalar(query_string.as_str())
        .fetch_one(&pool)
        .await
        .map_err(internal_error);

    match count_result {
        Ok(count_op) => match count_op {
            Some(count) => {
                if count == 0 {
                    return Ok(Json(SetObjectsResonse {
                        count,
                        success: true,
                    }));
                }
                return Ok(Json(SetObjectsResonse {
                    count,
                    success: false,
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
struct GetObjectsResponse {
    objects: Vec<LevelObject>,
}

#[derive(Serialize, Deserialize)]
struct GetObjectByIdResonse {
    object: LevelObject,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetLevelObjectRequest {
    version: String,
    object_type: String,
    position: String,
    rotation: String,
    scale: String,
    collider: String,
}
#[derive(Serialize, Deserialize)]
struct SetObjectsResonse {
    count: i64,
    success: bool,
}
