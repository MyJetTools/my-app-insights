use hyper::Uri;
use my_telemetry::MyTelemetry;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio::sync::Mutex;

use super::TelemetryEvent;

pub struct ReceiverStructure {
    pub receiver: tokio::sync::mpsc::UnboundedReceiver<TelemetryEvent>,
    pub key: String,
}

pub struct AppInsightsTelemetry {
    pub initialized: AtomicBool,
    pub publisher: tokio::sync::mpsc::UnboundedSender<TelemetryEvent>,
    pub receiver: Mutex<Option<ReceiverStructure>>,
}

impl AppInsightsTelemetry {
    pub fn new() -> Self {
        let app_insights_key = std::env::var("APPINSIGHTS_INSTRUMENTATIONKEY");

        let (transactions_sender, transactions_receiver) = tokio::sync::mpsc::unbounded_channel();

        let receiver = if let Ok(app_insights_key) = app_insights_key {
            Some(ReceiverStructure {
                key: app_insights_key,
                receiver: transactions_receiver,
            })
        } else {
            None
        };

        Self {
            publisher: transactions_sender,
            receiver: Mutex::new(receiver),
            initialized: AtomicBool::new(false),
        }
    }

    pub fn write_http_request_duration(
        &self,
        url: Uri,
        method: hyper::Method,
        status_code: u16,
        duration: Duration,
    ) {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let result = self.publisher.send(TelemetryEvent::HttpServerEvent {
            url,
            status_code,
            duration,
            method,
        });

        if let Err(err) = result {
            println!("Can not send telemetry event: {}", err)
        }
    }

    pub fn write_dependency_request_duration(
        &self,
        name: String,
        dependency_type: String,
        target: String,
        success: bool,
        duration: Duration,
    ) {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let result = self.publisher.send(TelemetryEvent::HttpDependencyEvent {
            name,
            dependency_type,
            target,
            duration,
            success,
        });

        if let Err(err) = result {
            println!("Can not send telemetry event: {}", err)
        }
    }

    pub async fn start(&self, shutting_down: Arc<AtomicBool>) {
        let result = {
            let mut write_access = self.receiver.lock().await;
            let mut result = None;

            std::mem::swap(&mut result, &mut write_access);
            result
        };

        match result {
            Some(result) => {
                println!("Application insights is plugged");

                self.initialized
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                super::telemetry_publisher::start(shutting_down, result.key, result.receiver).await;
            }
            None => {
                println!("Application insights is not plugged");
                let duration = Duration::from_secs(1);
                while !shutting_down.load(std::sync::atomic::Ordering::Relaxed) {
                    tokio::time::sleep(duration).await;
                }
            }
        }
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
