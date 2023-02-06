use std::{future::ready, net::SocketAddr, time::Instant};

use axum::{
    extract::MatchedPath,
    http::Request,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "imagemagick")]
mod imagemagick;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_tracing_aka_logging=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let recorder_handle = setup_metrics_recorder();

    // build our application with a route
    let app = Router::new()
        .route("/", post(routes::root))
        // .route("/img/info", post(routes::img_info))
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .route_layer(middleware::from_fn(track_metrics))
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 8763));
    println!("listening on http://{}/", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!("http_requests_duration_seconds", latency, &labels);

    response
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}
