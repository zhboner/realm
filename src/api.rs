use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{delete, get, post, put},
    Router,
    middleware::from_fn_with_state,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use headers::HeaderName;

use crate::conf::{EndpointConf, EndpointInfo, Config, NetConf};
use realm_core::tcp::run_tcp;
use realm_core::udp::run_udp;

static X_API_KEY: HeaderName = HeaderName::from_static("x-api-key");

async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    if state.api_key.is_none() {
        return Ok(next.run(request).await);
    }

    if let Some(api_key_header) = headers.get(&X_API_KEY) {
        if let Ok(provided_key) = api_key_header.to_str() {
            if let Some(expected_key) = &state.api_key {
                if provided_key == expected_key {
                    return Ok(next.run(request).await);
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Instance {
    pub id: String,
    pub config: EndpointConf,
    pub status: InstanceStatus,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub enum InstanceStatus {
    Running,
    Stopped,
    Failed(String),
}

#[derive(Clone)]
pub struct AppState {
    pub instances: Arc<AsyncMutex<HashMap<String, InstanceData>>>,
    pub api_key: Option<String>,
}

pub struct InstanceData {
    pub instance: Instance,
    pub tcp_handle: Option<JoinHandle<()>>,
    pub udp_handle: Option<JoinHandle<()>>,
}

#[utoipa::path(
    get,
    path = "/instances",
    responses(
        (status = 200, description = "List all instances", body = Vec<Instance>)
    )
)]
async fn list_instances(State(state): State<AppState>) -> Json<Vec<Instance>> {
    let instances = state.instances.lock().await;
    let list: Vec<Instance> = instances.values().map(|data| data.instance.clone()).collect();
    Json(list)
}

#[utoipa::path(
    post,
    path = "/instances",
    request_body = EndpointConf,
    responses(
        (status = 201, description = "Instance created", body = Instance),
        (status = 400, description = "Invalid configuration")
    )
)]
async fn create_instance(
    State(state): State<AppState>,
    Json(config): Json<EndpointConf>,
) -> Result<Json<Instance>, StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let instance = Instance {
        id: id.clone(),
        config: config.clone(),
        status: InstanceStatus::Running,
    };

    let endpoint_info = config.build();
    let (tcp_handle, udp_handle) = match start_realm_endpoint(endpoint_info) {
        Ok(handles) => handles,
        Err(e) => {
            let mut instances = state.instances.lock().await;
            let failed_instance = Instance {
                status: InstanceStatus::Failed(e.to_string()),
                ..instance
            };
            instances.insert(id.clone(), InstanceData {
                instance: failed_instance.clone(),
                tcp_handle: None,
                udp_handle: None,
            });
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut instances = state.instances.lock().await;
    instances.insert(id, InstanceData {
        instance: instance.clone(),
        tcp_handle,
        udp_handle,
    });
    Ok(Json(instance))
}

#[utoipa::path(
    get,
    path = "/instances/{id}",
    params(("id" = String, Path, description = "Instance ID")),
    responses(
        (status = 200, description = "Instance found", body = Instance),
        (status = 404, description = "Instance not found")
    )
)]
async fn get_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Instance>, StatusCode> {
    let instances = state.instances.lock().await;
    if let Some(data) = instances.get(&id) {
        Ok(Json(data.instance.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(
    put,
    path = "/instances/{id}",
    params(("id" = String, Path, description = "Instance ID")),
    request_body = EndpointConf,
    responses(
        (status = 200, description = "Instance updated", body = Instance),
        (status = 404, description = "Instance not found")
    )
)]
async fn update_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(config): Json<EndpointConf>,
) -> Result<Json<Instance>, StatusCode> {
    let mut instances = state.instances.lock().await;
    if let Some(mut data) = instances.remove(&id) {
        if let Some(tcp_handle) = data.tcp_handle.take() {
            tcp_handle.abort();
        }
        if let Some(udp_handle) = data.udp_handle.take() {
            udp_handle.abort();
        }

        let endpoint_info = config.clone().build();
        let (tcp_handle, udp_handle) = match start_realm_endpoint(endpoint_info) {
            Ok(handles) => handles,
            Err(e) => {
                data.instance.status = InstanceStatus::Failed(e.to_string());
                data.tcp_handle = None;
                data.udp_handle = None;
                instances.insert(id.clone(), data);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        data.instance.config = config;
        data.instance.status = InstanceStatus::Running;
        data.tcp_handle = tcp_handle;
        data.udp_handle = udp_handle;
        let instance = data.instance.clone();
        instances.insert(id, data);
        Ok(Json(instance))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(
    post,
    path = "/instances/{id}/start",
    params(("id" = String, Path, description = "Instance ID")),
    responses(
        (status = 200, description = "Instance started", body = Instance),
        (status = 404, description = "Instance not found"),
        (status = 409, description = "Instance already running")
    )
)]
async fn start_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Instance>, StatusCode> {
    let mut instances = state.instances.lock().await;
    if let Some(mut data) = instances.remove(&id) {
        if data.tcp_handle.is_some() || data.udp_handle.is_some() {
            if matches!(data.instance.status, InstanceStatus::Running) {
                instances.insert(id, data);
                return Err(StatusCode::CONFLICT);
            }
        }

        let endpoint_info = data.instance.config.clone().build();
        let (tcp_handle, udp_handle) = match start_realm_endpoint(endpoint_info) {
            Ok(handles) => handles,
            Err(e) => {
                data.instance.status = InstanceStatus::Failed(e.to_string());
                data.tcp_handle = None;
                data.udp_handle = None;
                instances.insert(id.clone(), data);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        data.instance.status = InstanceStatus::Running;
        data.tcp_handle = tcp_handle;
        data.udp_handle = udp_handle;
        let instance = data.instance.clone();
        instances.insert(id, data);
        Ok(Json(instance))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(
    post,
    path = "/instances/{id}/stop",
    params(("id" = String, Path, description = "Instance ID")),
    responses(
        (status = 200, description = "Instance stopped", body = Instance),
        (status = 404, description = "Instance not found"),
        (status = 409, description = "Instance already stopped")
    )
)]
async fn stop_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Instance>, StatusCode> {
    let mut instances = state.instances.lock().await;
    if let Some(mut data) = instances.remove(&id) {
        if data.tcp_handle.is_none() && data.udp_handle.is_none() {
            if !matches!(data.instance.status, InstanceStatus::Running) {
                instances.insert(id, data);
                return Err(StatusCode::CONFLICT);
            }
        }

        if let Some(tcp_handle) = data.tcp_handle.take() {
            tcp_handle.abort();
        }
        if let Some(udp_handle) = data.udp_handle.take() {
            udp_handle.abort();
        }

        data.instance.status = InstanceStatus::Stopped;
        let instance = data.instance.clone();
        instances.insert(id, data);
        Ok(Json(instance))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(
    post,
    path = "/instances/{id}/restart",
    params(("id" = String, Path, description = "Instance ID")),
    responses(
        (status = 200, description = "Instance restarted", body = Instance),
        (status = 404, description = "Instance not found")
    )
)]
async fn restart_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Instance>, StatusCode> {
    let mut instances = state.instances.lock().await;
    if let Some(mut data) = instances.remove(&id) {
        if let Some(tcp_handle) = data.tcp_handle.take() {
            tcp_handle.abort();
        }
        if let Some(udp_handle) = data.udp_handle.take() {
            udp_handle.abort();
        }

        let endpoint_info = data.instance.config.clone().build();
        let (tcp_handle, udp_handle) = match start_realm_endpoint(endpoint_info) {
            Ok(handles) => handles,
            Err(e) => {
                data.instance.status = InstanceStatus::Failed(e.to_string());
                data.tcp_handle = None;
                data.udp_handle = None;
                instances.insert(id.clone(), data);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        data.instance.status = InstanceStatus::Running;
        data.tcp_handle = tcp_handle;
        data.udp_handle = udp_handle;
        let instance = data.instance.clone();
        instances.insert(id, data);
        Ok(Json(instance))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(
    delete,
    path = "/instances/{id}",
    params(("id" = String, Path, description = "Instance ID")),
    responses(
        (status = 204, description = "Instance deleted"),
        (status = 404, description = "Instance not found")
    )
)]
async fn delete_instance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut instances = state.instances.lock().await;
    if let Some(data) = instances.remove(&id) {
        if let Some(tcp_handle) = data.tcp_handle {
            tcp_handle.abort();
        }
        if let Some(udp_handle) = data.udp_handle {
            udp_handle.abort();
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn start_realm_endpoint(endpoint_info: EndpointInfo) -> std::io::Result<(Option<JoinHandle<()>>, Option<JoinHandle<()>>)> {
    let EndpointInfo {
        endpoint,
        no_tcp,
        use_udp,
    } = endpoint_info;

    let mut tcp_handle = None;
    let mut udp_handle = None;

    if use_udp {
        let endpoint_clone = endpoint.clone();
        udp_handle = Some(tokio::spawn(async move {
            if let Err(e) = run_udp(endpoint_clone).await {
                log::error!("UDP endpoint failed: {}", e);
            }
        }));
    }

    if !no_tcp {
        tcp_handle = Some(tokio::spawn(async move {
            if let Err(e) = run_tcp(endpoint).await {
                log::error!("TCP endpoint failed: {}", e);
            }
        }));
    }

    Ok((tcp_handle, udp_handle))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_instances,
        create_instance,
        get_instance,
        update_instance,
        delete_instance,
        start_instance,
        stop_instance,
        restart_instance,
    ),
    components(
        schemas(Instance, InstanceStatus, EndpointConf, NetConf)
    ),
    tags(
        (name = "realm", description = "Realm instance management API")
    )
)]
struct ApiDoc;

pub async fn start_api_server(port: u16, api_key: Option<String>) {
    let state = AppState {
        instances: Arc::new(AsyncMutex::new(HashMap::new())),
        api_key: api_key.clone(),
    };

    let app = Router::new()
        .route("/instances", get(list_instances))
        .route("/instances", post(create_instance))
        .route("/instances/:id", get(get_instance))
        .route("/instances/:id", put(update_instance))
        .route("/instances/:id", delete(delete_instance))
        .route("/instances/:id/start", post(start_instance))
        .route("/instances/:id/stop", post(stop_instance))
        .route("/instances/:id/restart", post(restart_instance))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    if let Some(key) = &api_key {
        println!("Starting API server on {} with authentication enabled", addr);
        println!("API Key: {}", key);
    } else {
        println!("Starting API server on {} without authentication", addr);
    }
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
