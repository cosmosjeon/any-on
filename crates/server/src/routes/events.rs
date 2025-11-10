use axum::{
    BoxError, Router,
    extract::State,
    middleware::from_fn_with_state,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
    routing::get,
};
use deployment::Deployment;
use futures_util::TryStreamExt;

use crate::{DeploymentImpl, middleware::auth::require_auth};

pub async fn events(
    State(deployment): State<DeploymentImpl>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, BoxError>>>, axum::http::StatusCode>
{
    // Ask the container service for a combined "history + live" stream
    let stream = deployment.stream_events().await;
    Ok(Sse::new(stream.map_err(|e| -> BoxError { e.into() })).keep_alive(KeepAlive::default()))
}

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    let events_router = Router::new()
        .route("/", get(events))
        .layer(from_fn_with_state(deployment.clone(), require_auth));

    Router::new().nest("/events", events_router)
}
