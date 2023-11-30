# A Stupidly Simple Load Balancer (SSLB)
*Created by Jaeden Quintana*

A simple load balancer which handles incoming HTTP requests and redirects their traffic
to a registered endpoint.

## How to Use
The executable reads a TOML file named "sslb.toml" which contains the configuration
for the load balancer. Below is an example of how the config file should be formatted.

```
[config]
addr = "1.2.3.4:80" # The IP addr of the server and the port to run on.
endpoints = [ # The IP addr of the endpoints and their ports.
  "1.2.3.5:80",
  "1.2.7.8:80", 
  ...
]
```

## Supported Policies
* Round-robin
* Random
