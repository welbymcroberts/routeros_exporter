use std::collections::HashMap;
use std::error::Error;

pub fn collector_preamble() -> (std::time::Instant, String, u8, String) {
    let now = std::time::Instant::now();
    let ret = "".to_string();
    let labels: String = "".to_owned();
    (now, ret, 0, labels)
}

pub async fn collector_request_get(
    url: String,
    check_ssl: bool,
) -> Result<serde_json::Value, reqwest::Error> {
    let request = reqwest::Client::builder()
        .danger_accept_invalid_certs(!check_ssl)
        .build()
        .unwrap()
        .get(url)
        .send()
        .await?;

    match request.status() {
        reqwest::StatusCode::OK => {
            let json: serde_json::Value = request.json().await?;
            Ok(json)
        }
        _ => Ok(serde_json::json!(null)),
    }
}

pub async fn collector_request_post(
    url: String,
    data: HashMap<&str, &str>,
    check_ssl: bool,
) -> Result<serde_json::Value, reqwest::Error> {
    let request = reqwest::Client::builder()
        .danger_accept_invalid_certs(!check_ssl)
        .build()
        .unwrap()
        .post(url)
        .json(&data)
        .send()
        .await?;

    match request.status() {
        reqwest::StatusCode::OK => {
            let json: serde_json::Value = request.json().await?;
            Ok(json)
        }
        _ => Ok(serde_json::json!(null)),
    }
}

pub fn make_metric(prefix: &str, name: &str, labels: &str, value: &str) -> String {
    let mut r = "".to_owned();

    let metric_name_full = format!("{}_{}", prefix, str::replace(&*name, "-", "_"));
    r = format!("{}{}{{{}}} {}\n", r, metric_name_full, labels, value);

    r
}

pub fn make_metric_preamble(
    prefix: &str,
    name: &str,
    metric_type: &str,
    help: &str,
    unit: &str,
) -> String {
    let metric_name_full = format!("{}_{}", prefix, str::replace(&*name, "-", "_"));

    let mut r = "".to_owned();

    r = format!("{}# HELP {} {}\n", r, metric_name_full, help);
    r = format!("{}# TYPE {} {}\n", r, metric_name_full, metric_type);
    if unit != "" {
        r = format!("{}# UNIT {} {}\n", r, metric_name_full, unit);
    }

    r
}

pub fn convert_to_bps(original: &str) -> String {
    return if original.ends_with("Gbps") {
        (original.replace("Gbps", "").parse::<f64>().unwrap_or(-1.0) * 1000.0 * 1000.0 * 1000.0).to_string()
    } else if original.ends_with("Mbps") {
        (original.replace("Mbps", "").parse::<f64>().unwrap_or(-1.0) * 1000.0 * 1000.0).to_string()
    } else {
        0.to_string()
    };
}
