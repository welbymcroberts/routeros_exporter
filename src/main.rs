#[macro_use]
extern crate lazy_static;

use futures;
use secrecy::ExposeSecret;
use std::convert::Infallible;
use std::hash::Hash;
use std::net::SocketAddr;
// use std::time::Instant;
use tokio;
use warp::Filter;

use routeros_exporter::configuration::{get_configuration, Collectors};

// lazy_static the config
lazy_static! {
    static ref CONFIG: routeros_exporter::configuration::Settings =
        get_configuration().expect("Failed to read configuration.");
}

// HTTP GET /probe
async fn http_get_probe(q: Vec<(String, String)>) -> Result<impl warp::Reply, Infallible> {
    // Each collector will be a new future
    let tasks = futures::stream::FuturesUnordered::new();

    // Clone default config
    let mut c = CONFIG.defaults.collectors.clone().unwrap();
    let mut port = CONFIG.defaults.port;
    let mut username = CONFIG.defaults.username.clone();
    let mut password = CONFIG.defaults.password.expose_secret().clone();
    let mut address = CONFIG.defaults.address.clone();
    let mut check_ssl = CONFIG.defaults.check_ssl;

    // var for results to be added to
    let mut ret = "".to_owned();

    // If target in query, set it now
    for (k, v) in q.clone() {
        match k.as_str() {
            "target" => address = v.as_str().parse()?,
            _ => {}
        }
    }

    // If the target is defined in config, override
    if CONFIG.instances.is_some() {
        let i = CONFIG.instances.as_ref();
        for instance in i.unwrap() {
            if instance.address.as_str() == address {
                c = instance.collectors.as_ref().unwrap().clone();
                username = instance.username.clone();
                password = instance.password.expose_secret().clone();
                port = instance.port;
                check_ssl = instance.check_ssl;
            }
        }
    }

    // Try and match query params
    // not using Struct as collectors could be defined multiple times, once per collector
    for (k, v) in q {
        match k.as_str() {
            "collectors" => match v.as_str() {
                "ip_firewall" => {
                    c.ip_firewall = Some(true);
                }
                "ip_firewall_filter" => {
                    c.ip_firewall_filter = Some(true);
                }
                "ip_firewall_nat" => {
                    c.ip_firewall_nat = Some(true);
                }
                "ip_firewall_mangle" => {
                    c.ip_firewall_mangle = Some(true);
                }
                "health" => {
                    c.health = Some(true);
                }
                "resources" => {
                    c.resources = Some(true);
                }
                "interfaces" => {
                    c.interfaces = Some(true);
                }
                "interfaces_poe" => {
                    c.interfaces_poe = Some(true);
                }
                "interfaces_monitor" => {
                    c.interfaces_monitor = Some(true);
                }
                _ => {}
            },
            "check_ssl" => {
                check_ssl = true;
            }
            //  Removing as this shouldn't be over the wire
            // "username" => {
            //     username = v.as_str().parse()?;
            // }
            // "password" => {
            //     password = v;
            // }
            "port" => {
                port = v.as_str().parse().unwrap();
            }
            _ => {}
        }
    }

    if c.interfaces == Some(true) {
        tasks.push(routeros_exporter::spawn_collector!(
            routeros_exporter::collectors::interfaces::run,
            (*username).parse()?,
            (*password).parse()?,
            (*address).parse()?,
            port.clone(),
            check_ssl,
            CONFIG.clone()
        ));
    }
    if c.interfaces_monitor == Some(true) {
        tasks.push(routeros_exporter::spawn_collector!(
            routeros_exporter::collectors::interfaces_monitor::run,
            (*username).parse()?,
            (*password).parse()?,
            (*address).parse()?,
            port.clone(),
            check_ssl,
            CONFIG.clone()
        ));
    }
    if c.interfaces_poe == Some(true) {
        tasks.push(routeros_exporter::spawn_collector!(
            routeros_exporter::collectors::interfaces_poe::run,
            (*username).parse()?,
            (*password).parse()?,
            (*address).parse()?,
            port.clone(),
            check_ssl,
            CONFIG.clone()
        ));
    }

    if c.ip_firewall == Some(true) {
        if c.ip_firewall_filter == Some(true) {
            tasks.push(routeros_exporter::spawn_collector!(
            routeros_exporter::collectors::ip_firewall_filter::run,
            (*username).parse()?,
            (*password).parse()?,
            (*address).parse()?,
            port.clone(),
            check_ssl,
            CONFIG.clone()
        ));
        }
        if c.ip_firewall_nat == Some(true) {
            tasks.push(routeros_exporter::spawn_collector!(
            routeros_exporter::collectors::ip_firewall_nat::run,
            (*username).parse()?,
            (*password).parse()?,
            (*address).parse()?,
            port.clone(),
            check_ssl,
            CONFIG.clone()
        ));
        }
        if c.ip_firewall_mangle == Some(true) {
            tasks.push(routeros_exporter::spawn_collector!(
            routeros_exporter::collectors::ip_firewall_mangle::run,
            (*username).parse()?,
            (*password).parse()?,
            (*address).parse()?,
            port.clone(),
            check_ssl,
            CONFIG.clone()
        ));
        }
        if c.ip_firewall_raw == Some(true) {
            tasks.push(routeros_exporter::spawn_collector!(
            routeros_exporter::collectors::ip_firewall_raw::run,
            (*username).parse()?,
            (*password).parse()?,
            (*address).parse()?,
            port.clone(),
            check_ssl,
            CONFIG.clone()
        ));
        }
    }

    // for each task, await
    for t in tasks {
        // TODO: handle connection refused etc
        ret = format!("{}{}\n", ret, t.await.unwrap().unwrap());

        // TODO: handle probe status.
        // TODO: Time taken
    }

    Ok(warp::http::Response::builder()
        .header("Content-Type", "text/plain")
        .body(ret))
}

// HTTP GET /
async fn http_get_root() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::http::Response::builder()
        .header("Content-Type", "text/plain")
        .body("routeros_exporter running. This needs to be called at /probe"))
}
#[tokio::main]
async fn main() {
    // Routes
    // Root,
    let root = warp::path::end().and_then(http_get_root);
    // /probe?<query>
    let probe = warp::path("probe")
        .and(warp::query::<Vec<(String, String)>>())
        .and_then(http_get_probe);
    // Combine the above to a group of routes
    let routes = warp::get().and(root.or(probe));
    // Build listen address from config
    // TODO: address to bind to
    let addr = SocketAddr::from(([0, 0, 0, 0], CONFIG.server.port));
    // Serve up 'routes' on 'addr'
    warp::serve(routes).run(addr).await;
}
