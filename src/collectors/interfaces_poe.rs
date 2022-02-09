use crate::collectors::helpers::{
    collector_preamble, collector_request_get, collector_request_post, make_metric,
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
            username, password, address, port, "rest/interface/ethernet/poe?.proplist=.id"
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
                username, password, address, port, "rest/interface/ethernet/poe/monitor"
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

                // Only do preamble on first interface
                let mut preamble: bool = false;
                if count != 0 {
                    preamble = true
                }

                // Get Status of interface POE
                let mut status: u8 = 255;
                match interface.get("poe-out-status").unwrap().as_str().unwrap() {
                    "powered-on" => status = 0,
                    "waiting-for-load" => status = 1,
                    "short-circuit" => status = 2,
                    "overload" => status = 3,
                    "voltage-too-low" => status = 4,
                    "current-too-low" => status = 5,
                    "off" => status = 6,
                    "disabled" => status = 254,
                    _ => status = 255,
                };

                // Interface Running
                ret = format!(
                    "{}{}",
                    ret,
                    metrics(
                        &config.metrics_prefix,
                        "interface_poe_status",
                        &status.to_string(),
                        &labels.to_string(),
                        "gauge",
                        "PoE Output Status. { 0: powered-on, 1: waiting-for-load, 2: short-circuit, 3: overload, 4: voltage-too-low, 5: current-too-low, 6: off, 254: disabled, 255: unknown }",
                        "Status",
                        preamble
                    )
                );

                // Simple metrics
                let metrics_hash = HashMap::from([
                    (
                        "poe-out-voltage",
                        vec![
                            "interface_poe_out_voltage",
                            "PoE Output Voltage",
                            "volts",
                            "gauge",
                        ],
                    ),
                    (
                        "poe-out-current",
                        vec![
                            "interface_poe_out_current",
                            "PoE Output Current",
                            "amps",
                            "gauge",
                        ],
                    ),
                    (
                        "poe-out-power",
                        vec![
                            "interface_poe_out_power",
                            "PoE Output Power",
                            "watts",
                            "gauge",
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

                // // Mapping Metrics
                // let mapped_metrics_hash = HashMap::from([
                //     ("none", vec!("", "", "", "", ""))
                // ]);
                // // Iterate over metrics_hash
                // for (metric,metric_attr) in mapped_metrics_hash {
                //     if interface.get(metric) != None {
                //         // match based on metric name
                //         let mut value = "";
                //         match metric {
                //             _ => {}
                //         };
                //
                //         ret = format!(
                //             "{}{}",
                //             ret,
                //             metrics(
                //                 &config.metrics_prefix,
                //                 metric_attr[0],
                //                 value,
                //                 &labels.to_string(),
                //                 metric_attr[3],
                //                 metric_attr[1],
                //                 metric_attr[2],
                //                 preamble
                //             )
                //         )
                //     }
                // };

                // Increase count
                count += 1;
            }
        }
    }

    Ok(ret) //"".to_string()
}
