use crate::{deployment, multi_agent, road_sign, stereo};
use axum::{
    extract::{DefaultBodyLimit, Json},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

type ApiResponse = Result<Json<Value>, (StatusCode, Json<ApiError>)>;

fn respond<T: Serialize>(result: Result<T, String>) -> ApiResponse {
    result
        .map(|x| Json(serde_json::to_value(x).expect("serializable result")))
        .map_err(|message| (StatusCode::BAD_REQUEST, Json(ApiError { error: message })))
}

async fn health() -> Json<Value> {
    Json(json!({"status":"ok","version":env!("CARGO_PKG_VERSION")}))
}

async fn experiments() -> Json<Value> {
    Json(json!({
        "experiments": [
            {
                "id": "stereo",
                "endpoint": "/api/v1/experiments/stereo",
                "question": "Can two observations be explained by one rectified physical point?"
            },
            {
                "id": "multi-agent",
                "endpoint": "/api/v1/experiments/multi-agent",
                "question": "Do independently feasible paths satisfy joint collision and deadline constraints?"
            },
            {
                "id": "deployment",
                "endpoint": "/api/v1/experiments/deployment",
                "question": "Can individually compatible versions preserve in-flight work across a time-bearing change?"
            },
            {
                "id": "road-sign",
                "endpoint": "/api/v1/experiments/road-sign",
                "question": "Does recognition occur early enough for an admissible response?"
            },
            {
                "id": "road-sign/sweep",
                "endpoint": "/api/v1/experiments/road-sign/sweep",
                "question": "How does sign design change the fraction of feasible operating conditions?"
            }
        ]
    }))
}

async fn stereo_endpoint(Json(input): Json<stereo::StereoInput>) -> ApiResponse {
    respond(stereo::run(input))
}

async fn multi_agent_endpoint(Json(input): Json<multi_agent::MultiAgentInput>) -> ApiResponse {
    respond(multi_agent::run(input))
}

async fn deployment_endpoint(Json(input): Json<deployment::DeploymentInput>) -> ApiResponse {
    respond(deployment::run(input))
}

async fn road_sign_endpoint(Json(input): Json<road_sign::RoadSignInput>) -> ApiResponse {
    respond(road_sign::run(input))
}

async fn road_sign_sweep_endpoint(Json(input): Json<road_sign::RoadSignSweepInput>) -> ApiResponse {
    respond(road_sign::sweep(input))
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/experiments", get(experiments))
        .route("/api/v1/experiments/stereo", post(stereo_endpoint))
        .route("/api/v1/experiments/multi-agent", post(multi_agent_endpoint))
        .route("/api/v1/experiments/deployment", post(deployment_endpoint))
        .route("/api/v1/experiments/road-sign", post(road_sign_endpoint))
        .route("/api/v1/experiments/road-sign/sweep", post(road_sign_sweep_endpoint))
        .layer(DefaultBodyLimit::max(64 * 1024))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_works() {
        let response = router().oneshot(
            Request::builder().uri("/health").body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn stereo_accepts_empty_json_as_documented_default() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/experiments/stereo")
            .header("content-type", "application/json")
            .body(Body::from("{}")).unwrap();
        let response = router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 100_000).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["correspondence_admissible"], true);
    }

    #[tokio::test]
    async fn invalid_stereo_values_return_400() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/experiments/stereo")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"baseline_m":-1}"#)).unwrap();
        let response = router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn multi_agent_api_returns_feasible_joint_schedule() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/experiments/multi-agent")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&multi_agent::MultiAgentInput::default()).unwrap()))
            .unwrap();
        let response = router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 100_000).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["independent_collision"], true);
        assert_eq!(value["joint_schedule_found"], true);
    }
    #[tokio::test]
    async fn deployment_api_exposes_in_flight_incompatibility() {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/experiments/deployment")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let response = router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 100_000).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["initial_configuration_compatible"], true);
        assert_eq!(value["final_configuration_compatible"], true);
        assert_eq!(value["schema_rejected"], 1);
    }

}
