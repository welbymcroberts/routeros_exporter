#[macro_export]
macro_rules! spawn_collector {
    ($collector:expr, $username:expr, $password:expr, $address:expr, $port:expr, $check_ssl:expr, $config:expr) => {
        tokio::spawn($collector(
            $username, $password, $address, $port, $check_ssl, $config,
        ));
    };
}
