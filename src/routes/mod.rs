use crate::model::ServiceSchema;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use opentelemetry::trace::TraceContextExt;
use serde::Serialize;
use tracing::{info, span, Instrument, Level};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Serialize)]
struct Health {
    healthy: bool,
}

pub(crate) async fn health() -> impl IntoResponse {
    let health = Health { healthy: true };

    (StatusCode::OK, Json(health))
}

pub(crate) async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

pub(crate) async fn graphql_handler(
    req: GraphQLRequest,
    Extension(schema): Extension<ServiceSchema>,
) -> GraphQLResponse {
    let span = span!(Level::INFO, "graphql_execution"); // (1)

    info!("Processing GraphQL request");

    let response = async move { schema.execute(req.into_inner()).await } // (2)
        .instrument(span.clone())
        .await;

    info!("Processing GraphQL request finished");

    response
        .extension( // (3)
            "traceId",
            async_graphql::Value::String(format!(
                "{}",
                span.context().span().span_context().trace_id()
            )),
        )
        .into()
}
