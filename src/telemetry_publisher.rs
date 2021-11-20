use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use appinsights::{
    telemetry::{RemoteDependencyTelemetry, RequestTelemetry},
    TelemetryClient, TelemetryConfig,
};
use tokio::sync::mpsc::UnboundedReceiver;

use super::TelemetryEvent;

pub async fn start(
    shutting_down: Arc<AtomicBool>,
    app_insights_key: String,
    mut events_receiver: UnboundedReceiver<TelemetryEvent>,
) {
    // configure telemetry config with custom settings
    let config = TelemetryConfig::builder()
        // provide an instrumentation key for a client
        .i_key(app_insights_key)
        // set a new maximum time to wait until data will be sent to the server
        .interval(Duration::from_secs(5))
        // construct a new instance of telemetry configuration
        .build();

    // configure telemetry client with default settings
    let mut client = TelemetryClient::from_config(config);

    client
        .context_mut()
        .tags_mut()
        .cloud_mut()
        .set_role("my_no_sql_server".to_string());

    while let Some(event) = events_receiver.recv().await {
        if shutting_down.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        match event {
            TelemetryEvent::HttpServerEvent {
                url,
                status_code,
                duration,
                method,
            } => {
                let telemetry =
                    RequestTelemetry::new(method, url, duration, status_code.to_string());

                client.track(telemetry);
            }
            TelemetryEvent::HttpDependencyEvent {
                name,
                dependency_type,
                target,
                duration,
                success,
            } => {
                let telemetry = RemoteDependencyTelemetry::new(
                    name,
                    dependency_type,
                    duration,
                    target,
                    success,
                );
                client.track(telemetry);
            }
        }
    }
}
