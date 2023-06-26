use lazy_static::lazy_static;
use opentelemetry::sdk::trace::{self, RandomIdGenerator, Sampler};
use opentelemetry_otlp::WithExportConfig;
use reqwest::header;
use std::{collections::HashMap, time::Duration};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{fmt::MakeWriter, Layer};

use crate::json;

lazy_static! {
    static ref LOGGER: (NonBlocking, WorkerGuard) = tracing_appender::non_blocking(Logger::new());
}

pub struct Logger {
    client: reqwest::blocking::Client,
}

impl Logger {
    fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        let mut auth_value = header::HeaderValue::from_str(
            &format!(
                "Basic {}",
                std::env::var("OPEN_OBSERVE_TOKEN").expect("missing OPEN_OBSERVE_TOKEN env var")
            )
            .to_string(),
        )
        .unwrap();
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        let client = tokio::task::block_in_place(|| {
            reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap()
        });
        Self { client }
    }
}

impl std::io::Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let data = buf.to_vec();
        let _ = tokio::task::block_in_place(|| {
            self.client.post("https://api.openobserve.ai/api/jocelyn_organization_1899_76kO6L8JBe7b61O/logs/_multi").body(data).send()
        });
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl MakeWriter<'_> for Logger {
    type Writer = Self;

    fn make_writer(&'_ self) -> Self::Writer {
        Self {
            client: self.client.clone(),
        }
    }
}

pub fn init_logger<S>() -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    let mut headers = header::HeaderMap::new();
    let mut auth_value = header::HeaderValue::from_str(
        &format!(
            "Basic {}",
            std::env::var("OPEN_OBSERVE_TOKEN").expect("missing OPEN_OBSERVE_TOKEN env var")
        )
        .to_string(),
    )
    .unwrap();
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let exporter = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_http_client(exporter)
                    .with_endpoint("https://api.openobserve.ai/api/jocelyn_organization_1899_76kO6L8JBe7b61O/traces")
                    .with_timeout(Duration::from_secs(3)),
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_max_events_per_span(64)
                    .with_max_attributes_per_span(16)
                    .with_max_events_per_span(16),
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap();

    let layer = tracing_subscriber::fmt::layer()
        .json()
        .event_format(json::Json)
        .with_writer(LOGGER.0.clone());

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    telemetry.and_then(layer)
}
