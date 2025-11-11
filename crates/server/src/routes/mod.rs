use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{IntoMakeService, get},
};
use tower_http::cors::{CorsLayer, Any};

use crate::DeploymentImpl;

pub mod approvals;
pub mod auth;
pub mod config;
pub mod containers;
pub mod drafts;
pub mod events;
pub mod execution_processes;
pub mod filesystem;
pub mod frontend;
#[cfg(feature = "cloud")]
pub mod github;
pub mod health;
pub mod images;
pub mod projects;
pub mod tags;
pub mod task_attempts;
pub mod tasks;

pub fn router(deployment: DeploymentImpl) -> IntoMakeService<Router> {
    // Configure CORS
    // Default: Allow all origins for development
    // Set CORS_ALLOWED_ORIGINS env var to restrict (comma-separated list)
    let cors = if let Ok(origins) = std::env::var("CORS_ALLOWED_ORIGINS") {
        let allowed_origins: Vec<_> = origins
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().parse().expect("Invalid CORS origin"))
            .collect();

        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_credentials(true)
    } else {
        // Development mode: allow all origins
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Create routers with different middleware layers
    let routes = Router::new()
        .route("/health", get(health::health_check))
        .merge(config::router())
        .merge(containers::router(&deployment))
        .merge(projects::router(&deployment))
        .merge(drafts::router(&deployment))
        .merge(tasks::router(&deployment))
        .merge(task_attempts::router(&deployment))
        .merge(execution_processes::router(&deployment))
        .merge(tags::router(&deployment))
        .merge(auth::router(&deployment))
        .merge(filesystem::router(&deployment))
        .merge(events::router(&deployment))
        .merge(approvals::router(&deployment))
        .nest("/images", images::routes(&deployment))
        .layer(from_fn_with_state(
            deployment.clone(),
            auth::sentry_user_context_middleware,
        ));

    #[cfg(feature = "cloud")]
    let routes = routes.merge(github::github_router());

    #[cfg(not(feature = "cloud"))]
    let routes = routes;

    let base_routes = routes.with_state(deployment);

    Router::new()
        .route("/", get(frontend::serve_frontend_root))
        .route("/{*path}", get(frontend::serve_frontend))
        .nest("/api", base_routes)
        .layer(cors)
        .into_make_service()
}
