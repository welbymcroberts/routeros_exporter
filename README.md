## routeros_exporter

A rust based OpenMetrics (prometheus etc) exporter for Mikrotik RouterOS devices running the RouterOS 7.x using the REST
API.

This project is influenced by the work @nshttpd (and others) has done on their golang based exporter -
nshttpd/mikrotik-exporter

This was created originally as a challenge to create an async REST API client, and should be considered a work in
progress. Some of what is mentioned in this readme may also not be implemented currently.

This project builds on nshttpd/mikrotik-exporter and is similar to blackbox_exporter and other who use the /probe
endpoint to specify which target is to be scraped. A configuration file is used to set the username and password for
each device, and optionally a set of collectors that are to be enabled.

When a request is created, a new thread is created for each poll, with multiple polls running at once. This is done to
try and reduce the time for polls to complete, as the exporter may be remote, and indeed the other side of the world (
not recommended!) from a device it is polling

## Testing

This has been tested with a number of different Mikrotik Devices (CCR's, CRS's, wap60g's, RB4001/5009's) running 7.1,
7.1.1 and 7.2rc1, however I would recommend that this is tested before deployment.

## Building

This is a 'normal' rust style project which can be build using the `cargo` command. `cargo build --release` should build
a binary in the release/ directory.

## Configuration

Configuration is read in the following order

1. default.toml This file is included with the distribtuion and must currently be resident in the directory which
   routeros_exporter is running from

2. config/config.toml This file is a local configuration file that is optional, and will override what is in
   default.toml

3. Environment variables All configuration variables can be prefixed with ROUTEROS_ to override the configuration files.
   The seperator used is __.

An exmaple setup might be as follows

* default.toml

```
# As per the default.toml includied in the repository. Refer to this file for the contents
```

* config/config.toml

```
# This would create a username/password entry (and port/check_ssl) for 192.168.88.10. Overriding the [default] stanza
[[instances]]
username = "someuser"
password = "password"
check_ssl = false
address = "192.168.88.10"
port = 443

# This would override for 192.168.88.20
[[instances]]
username = "someuser"
password = "password"
check_ssl = false
address = "192.168.88.20"
port = 443
# This would also disable the interfaces_poe collector
[instances.collectors]
interfaces_poe = false
```

* Environment variable
  ```ROUTEROS_SERVER__PORT=12345```

## Security

Whilst rust it's self is a 'safe' language, that does not mean that this exporter is 'safe'. There has not been any
security tests run, nor is there any encryption of the configuration file or the metrics endpoint.

The code may also have security issues associated with it, however the code is written to ensure that the 'safe' rust is
used, with memory saftey guarantees. Again this does not however ensure that the code is 'safe'.

This software has no guarantees provided.
