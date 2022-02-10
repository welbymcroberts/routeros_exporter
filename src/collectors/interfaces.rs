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
            username, password, address, port, "rest/interface"
        ),
        check_ssl,
    )
    .await?;

    // Count of interfaces
    let mut count = 0;
    for interface in json.as_array().unwrap() {
        labels = "".to_string();
        // Inteface name added to labels
        labels.push_str(&*format!(
            "interface=\"{}\"",
            interface["name"].as_str().unwrap()
        ));

        let mut running = 0;
        if interface["running"].as_str().unwrap() == "true" {
            running = 1;
        }

        // If interface has a MAC Address, add it as a label
        if interface.get("mac-address") != None {
            match interface["mac-address"].as_str().unwrap() {
                "00:00:00:00:00:00" => {}
                _ => {
                    labels.push_str(&*format!(
                        ",mac_address=\"{}\"",
                        interface["mac-address"].as_str().unwrap()
                    ));
                }
            }
        };

        // If interface has a type, add it as a label
        if interface.get("type") != None {
            labels.push_str(&*format!(
                ",type=\"{}\"",
                interface["type"].as_str().unwrap()
            ));
        };

        // If interface has is a slave, add it as a label
        if interface.get("slave") != None {
            labels.push_str(&*format!(",slave=\"{}\"", "true"));
        };

        // If interface has a comment, add it as a label
        if interface.get("comment") != None {
            // Strip "'s from output
            let comment: String = interface["comment"].as_str().unwrap().replace("\"", "_");
            if comment != "" { labels.push_str(&*format!(",comment=\"{}\"", comment)); }
        };

        // Only do preamble on first interface
        let mut preamble: bool = false;
        if count != 0 {
            preamble = true
        }

        // Interface Running
        ret = format!(
            "{}{}",
            ret,
            metrics(
                &config.metrics_prefix,
                "interface_running",
                &running.to_string(),
                &labels.to_string(),
                "gauge",
                "Status",
                "Boolean",
                preamble
            )
        );

        // Simple metrics
        let metrics_hash = HashMap::from([
            (
                "actual-mtu",
                vec![
                    "interface_actual_mtu",
                    "Interface Actual MTU",
                    "bytes",
                    "gauge",
                ],
            ),
            (
                "l2mtu",
                vec!["interface_l2_mtu", "Interface Layer2 MTU", "bytes", "gauge"],
            ),
            (
                "tx-queue-drop",
                vec![
                    "interface_tx_queue_drop_total",
                    "Interface Drops on TX Queue",
                    "frames",
                    "counter",
                ],
            ),
            (
                "link-downs",
                vec![
                    "interface_link_downs_total",
                    "Interface Link Downs",
                    "downs",
                    "counter",
                ],
            ),
            // todo
            // (
            //     "disabled",
            //     vec![
            //         "interface_disabled",
            //         "Interface disabled",
            //         "status",
            //         "gauge",
            //     ],
            // ),
            (
                "fp-rx-byte",
                vec![
                    "interface_fp_rx_byte_total",
                    "Interface Fastpath RX Byte",
                    "bytes",
                    "counter",
                ],
            ),
            (
                "fp-tx-byte",
                vec![
                    "interface_fp_tx_byte_total",
                    "Interface Fastpath TX Byte",
                    "bytes",
                    "counter",
                ],
            ),
            (
                "tx-byte",
                vec![
                    "interface_tx_byte_total",
                    "Interface TX Byte",
                    "bytes",
                    "counter",
                ],
            ),
            (
                "rx-byte",
                vec![
                    "interface_rx_byte_total",
                    "Interface RX Byte",
                    "bytes",
                    "counter",
                ],
            ),
        ]);

        // Iterate over metrics_hash
        for (metric, metric_attr) in metrics_hash {
            if interface.get(metric) != None {
                ret = format!(
                    "{}{}",
                    ret,
                    metrics(
                        &config.metrics_prefix,
                        metric_attr[0],
                        &interface[metric].as_str().unwrap(),
                        &labels.to_string(),
                        metric_attr[3],
                        metric_attr[1],
                        metric_attr[2],
                        preamble
                    )
                )
            }
        }

        // Mapping Metrics
        let mapped_metrics_hash = HashMap::from([
            ("none", vec!["", "", "", "", ""]),
            (
                "mtu",
                vec![
                    "interface_mtu",
                    "Interface configured MTU",
                    "bytes",
                    "gauge",
                ],
            ),
        ]);
        // Iterate over metrics_hash
        for (metric, metric_attr) in mapped_metrics_hash {
            let mut value = "";
            if interface.get(metric) != None {
                // match based on metric name
                match metric {
                    "mtu" => {
                        if interface["mtu"] == "auto" {
                            value = "65534"
                        } else {
                            value = &interface[metric].as_str().unwrap()
                        }
                    }
                    _ => {}
                };

                ret = format!(
                    "{}{}",
                    ret,
                    metrics(
                        &config.metrics_prefix,
                        metric_attr[0],
                        value,
                        &labels.to_string(),
                        metric_attr[3],
                        metric_attr[1],
                        metric_attr[2],
                        preamble
                    )
                )
            }
        }

        // Increase count
        count += 1;
    }
    Ok(ret)
}
