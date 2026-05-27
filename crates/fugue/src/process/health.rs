use crate::process::lifecycle::HealthStatus;

pub async fn check_health(port: u16) -> HealthStatus {
    let url = format!("http://127.0.0.1:{}/", port);

    match reqwest::get(&url).await {
        Ok(_) => HealthStatus::Healthy,
        Err(e) => {
            if e.is_connect() {
                HealthStatus::Starting
            } else {
                // Any response (even error) means workerd is running
                HealthStatus::Healthy
            }
        }
    }
}
