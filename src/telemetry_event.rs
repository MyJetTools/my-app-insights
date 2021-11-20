use std::time::Duration;

use hyper::Uri;

pub enum TelemetryEvent {
    HttpServerEvent {
        url: Uri,
        status_code: u16,
        duration: Duration,
        method: hyper::Method,
    },
    HttpDependencyEvent {
        name: String,
        dependency_type: String,
        target: String,
        success: bool,
        duration: Duration,
    },
}
