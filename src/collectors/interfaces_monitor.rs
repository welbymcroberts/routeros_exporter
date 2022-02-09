use crate::collectors::helpers::{
    collector_preamble, collector_request_get, collector_request_post, convert_to_bps, make_metric,
    make_metric_preamble,
};
use std::collections::HashMap;
use std::error::Error;

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
            username, password, address, port, "rest/interface/ethernet?.proplist=.id"
        ),
        check_ssl,
    )
    .await?;

    // Interfaces to send in next request
    let mut interfaces: String = "".to_string();

    // For each of the interface IDs, add it to interface
    if json.as_array() != None {
        for i in json.as_array().unwrap() {
            if interfaces.len() > 0 {
                interfaces.push_str(",")
            };
            interfaces.push_str(&*i.get(".id").unwrap().to_string());
        }

        // Build hashmap that we'll send as JSON for next request
        let mut map = HashMap::new();
        map.insert("numbers", interfaces.as_str());
        map.insert("duration", "0.1s");
        map.insert("interval", "0.1s");

        let json_poe = collector_request_post(
            format!(
                "https://{}:{}@{}:{}/{}",
                username, password, address, port, "rest/interface/ethernet/monitor"
            ),
            map,
            check_ssl,
        )
        .await?;

        // Count of interfaces
        let mut count = 0;
        if json_poe.as_array() != None {
            for interface in json_poe.as_array().unwrap() {
                labels = "".to_string();
                // Inteface name added to labels
                labels.push_str(&*format!(
                    "interface=\"{}\"",
                    interface["name"].as_str().unwrap()
                ));
                if interface.get("mac-address") != None {
                    match interface["mac-address"].as_str().unwrap() {
                        "00:00:00:00:00:00" => {}
                        _ => {
                            labels.push_str(&*format!(
                                ",mac-address={}",
                                interface["mac-address"].as_str().unwrap()
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
                        "sfp-temperature",
                        vec![
                            "interface_sfp_temperature",
                            "SFP Temperature",
                            "celsius",
                            "gauge",
                        ],
                    ),
                    (
                        "sfp-tx-power",
                        vec!["interface_sfp_tx_power", "SFP TX Power", "dBm", "gauge"],
                    ),
                    (
                        "sfp-rx-power",
                        vec!["interface_sfp_rx_power", "SFP RX Power", "dBm", "gauge"],
                    ),
                    (
                        "sfp-tx-bias-current",
                        vec![
                            "interface_sfp_tx_bias_current",
                            "SFP TX Bias Current",
                            "mA",
                            "gauge",
                        ],
                    ),
                    (
                        "sfp-supply-voltage",
                        vec![
                            "interface_sfp_supply_voltage",
                            "SFP Supply voltage",
                            "volts",
                            "gauge",
                        ],
                    ),
                    (
                        "sfp-wavelength",
                        vec!["interface_sfp_wavelength", "SFP Wavelength", "nm", "gauge"],
                    ),
                ]);

                // TODO: if there's not been a preamble yet, help needs to be done
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
                let mapped_metrics_hash = HashMap::from([(
                    "rate",
                    vec!["interface_rate", "Interface Speed", "bps", "gauge"],
                )]);
                // Iterate over metrics_hash
                for (metric, metric_attr) in mapped_metrics_hash {
                    let mut value = "".to_owned();
                    if interface.get(metric) != None {
                        // match based on metric name
                        match metric {
                            "rate" => {
                                value = convert_to_bps(interface[metric].as_str().unwrap());
                            }
                            _ => {}
                        };
                        if value != "" {
                            ret = format!(
                                "{}{}",
                                ret,
                                metrics(
                                    &config.metrics_prefix,
                                    metric_attr[0],
                                    &value,
                                    &labels.to_string(),
                                    metric_attr[3],
                                    metric_attr[1],
                                    metric_attr[2],
                                    preamble
                                )
                            )
                        }
                    }
                }

                // Increase count
                count += 1;
            }
        }
    }

    Ok(ret)
}
