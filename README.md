## POC usage of [rust-metrics](https://github.com/posix4e/rust-metrics) in [tokio](http://tokio.rs)

Building up towards a metrics solution for integration into tokio. 

This crate is a simple http server with two routes. `/` route simply returns while `/slow` route performs a sum of 2000000 random `i32`. Metrics are implemented by wrapping rust-metrics structs with Futures in line with tokio asyncness :).
 
On each connection current metrics are spit out to `stdout`:
 
```
slow, 1 # number of slow requests executed
open, 0 # number of open connections
fast, 0 # number of fast requests executed
```

Also incuded is the [locustfile](locustfile.py) that can be used to test performance and accuarcy of metrics implementation.

```bash
# run server
cargo run --release

# install and run locust
pip install locustio
locust -f locustfile.py --host=http://127.0.0.1:8080

# go to localhost:8089 and start pounding
```