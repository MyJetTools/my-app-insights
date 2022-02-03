use std::{sync::Arc, time::Duration};

use appinsights::{
    telemetry::{RemoteDependencyTelemetry, RequestTelemetry},
    TelemetryClient, TelemetryConfig,
};

use rust_extensions::ApplicationStates;

use crate::events_queue::EventsQueue;

use super::TelemetryEvent;

pub async fn start(
    role_name: String,
    app_states: Arc<dyn ApplicationStates + Sync + Send + 'static>,
    app_insights_key: Option<String>,
    events: Arc<EventsQueue>,
) {
    if let Some(app_insights_key) = &app_insights_key {
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
            .set_role(role_name);

        write_loop(client, app_states, events).await;
    } else {
        let duration = Duration::from_secs(1);
        while !app_states.is_shutting_down() {
            tokio::time::sleep(duration).await;
        }
    }
}

async fn write_loop(
    client: TelemetryClient,
    app_states: Arc<dyn ApplicationStates + Sync + Send + 'static>,
    events: Arc<EventsQueue>,
) {
    let duration = Duration::from_secs(1);

    while !app_states.is_shutting_down() {
        match events.dequeue().await {
            Some(events) => {
                for event in events {
                    match event {
                        TelemetryEvent::HttpServerEvent {
                            url,
                            status_code,
                            duration,
                            method,
                        } => {
                            let telemetry = RequestTelemetry::new(
                                method,
                                url,
                                duration,
                                status_code.to_string(),
                            );

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
            None => {
                tokio::time::sleep(duration).await;
            }
        }
    }
}
