use tokio::sync::Mutex;

use crate::TelemetryEvent;

pub struct EventsQueue {
    events: Mutex<Vec<TelemetryEvent>>,
}

impl EventsQueue {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }
    pub async fn enqueue(&self, event: TelemetryEvent) {
        let mut write_access = self.events.lock().await;
        write_access.push(event);
    }

    pub async fn dequeue(&self) -> Option<Vec<TelemetryEvent>> {
        let mut write_access = self.events.lock().await;
        if write_access.len() == 0 {
            return None;
        }

        let mut result = Vec::new();
        std::mem::swap(&mut result, &mut *write_access);
        Some(result)
    }
}
