use std::collections::HashMap;
use std::error::Error;

use crate::collectors::helpers::{
    collector_preamble, collector_request_get, make_metric, make_metric_preamble,
};

pub fn metrics(
    metric_prefix: &str,
    name: &str,
    value: &str,
    labels: &str,
    metric_type: &str,
    help: &str,
    unit: &str,
    supress_preamble: bool,
) -> String {
    let mut ret = "".to_owned();

    if !supress_preamble {
        ret = format!(
            "{}{}",
            ret,
            make_metric_preamble(metric_prefix, name, metric_type, help, unit)
        );
    }
    ret = format!("{}{}", ret, make_metric(metric_prefix, name, labels, value));

    ret
}

pub async fn run(
    username: String,
    password: String,
    address: String,
    port: u16,
    check_ssl: bool,
    config: crate::configuration::Settings,
) -> Result<String, reqwest::Error> {
    // Preample for collectors
    let (now, mut ret, mut errored, mut labels) = collector_preamble();

    // Perform request
    let json = collector_request_get(
        format!(
            "https://{}:{}@{}:{}/{}",
            username, password, address, port, "rest/ip/firewall/raw"
        ),
        check_ssl,
    )
        .await?;
    // Count of rules
    let mut count = 0;
    if json.as_array() != None {
        for rule in json.as_array().unwrap() {
            labels = "".to_string();
            for (k, v) in rule.as_object().unwrap() {
                match k.as_str() {
                    "bytes" | "packets" | "comment" | ".id" | "log" | "log-prefix" | "invalid"
                    | "disabled" => (), // skip
                    _ => {
                        // add labels for everything else
                        if labels.len() > 0 {
                            labels.push_str(",")
                        };
                        labels.push_str(&*format!(
                            "{k}=\"{v}\"",
                            k = k.replace("-", "_"),
                            v = v.as_str().unwrap()
                        ));
                    }
                }
            }
            // Only do preamble on first interface
            let mut preamble: bool = false;
            if count != 0 {
                preamble = true
            }

            // Simple metrics
            let metrics_hash = HashMap::from([
                (
                    "packets",
                    vec![
                        "ip_firewall_raw_packets_total",
                        "Firewall rule packet count",
                        "packets",
                        "counter",
                    ],
                ),
                (
                    "bytes",
                    vec![
                        "ip_firewall_raw_bytes_total",
                        "Firewall rule byte count",
                        "bytes",
                        "counter",
                    ],
                ),
            ]);
            // Iterate over metrics_hash
            for (metric, metric_attr) in metrics_hash {
                if rule.get(metric) != None {
                    ret = format!(
                        "{}{}",
                        ret,
                        metrics(
                            &config.metrics_prefix,
                            metric_attr[0],
                            &rule[metric].as_str().unwrap(),
                            &labels.to_string(),
                            metric_attr[3],
                            metric_attr[1],
                            metric_attr[2],
                            preamble,
                        )
                    )
                }
            }
            // Increase count
            count += 1;
        }
    }
    Ok(ret)
}
