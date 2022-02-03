use hyper::Uri;
use my_telemetry::MyTelemetry;
use rust_extensions::ApplicationStates;
use std::{sync::Arc, time::Duration};

use crate::events_queue::EventsQueue;

use super::TelemetryEvent;

pub struct AppInsightsTelemetry {
    events: Arc<EventsQueue>,
    app_insights_key: Option<String>,
    role_name: String,
}

impl AppInsightsTelemetry {
    pub fn new(role_name: String) -> Self {
        let app_insights_key = if let Ok(key) = std::env::var("APPINSIGHTS_INSTRUMENTATIONKEY") {
            Some(key)
        } else {
            None
        };

        Self {
            events: Arc::new(EventsQueue::new()),
            app_insights_key,
            role_name,
        }
    }

    pub fn write_http_request_duration(
        &self,
        url: Uri,
        method: hyper::Method,
        status_code: u16,
        duration: Duration,
    ) {
        let event = TelemetryEvent::HttpServerEvent {
            url,
            status_code,
            duration,
            method,
        };

        let events = self.events.clone();

        tokio::spawn(async move {
            events.enqueue(event).await;
        });
    }

    pub fn write_dependency_request_duration(
        &self,
        name: String,
        dependency_type: String,
        target: String,
        success: bool,
        duration: Duration,
    ) {
        let event = TelemetryEvent::HttpDependencyEvent {
            name,
            dependency_type,
            target,
            duration,
            success,
        };

        let events = self.events.clone();

        tokio::spawn(async move {
            events.enqueue(event).await;
        });
    }

    pub async fn start(&self, app_statess: Arc<dyn ApplicationStates + Sync + Send + 'static>) {
        super::telemetry_writer::start(
            self.role_name.clone(),
            app_statess,
            self.app_insights_key.clone(),
            self.events.clone(),
        )
        .await;
    }
}

impl MyTelemetry for AppInsightsTelemetry {
    fn track_url_duration(
        &self,
        method: hyper::Method,
        uri: hyper::Uri,
        http_code: u16,
        duration: Duration,
    ) {
        self.write_http_request_duration(uri, method, http_code, duration);
    }

    fn track_dependency_duration(
        &self,
        host: String,
        protocol: String,
        resource: String,
        success: bool,
        duration: Duration,
    ) {
        self.write_dependency_request_duration(host, protocol, resource, success, duration);
    }
}
