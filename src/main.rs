extern crate futures;
extern crate futures_cpupool;
extern crate rustc_serialize;
extern crate tokio_minihttp;
extern crate tokio_proto;
extern crate tokio_service;
extern crate metrics;
extern crate rand;

use rand::Rng;

use std::io;

use std::collections::HashMap;
use futures::{BoxFuture, Future, future};
use futures_cpupool::CpuPool;
use tokio_minihttp::{Request, Response};
use tokio_proto::TcpServer;
use tokio_service::Service;

use std::time::SystemTime;

use std::sync::Arc;
use std::sync::RwLock;

use metrics::metrics::{StdGauge, Gauge};

struct Registry {
    map: RwLock<HashMap<String, Arc<StdGauge>>>
}

impl Registry {
    fn add(&self, key: &str) {
        let mut s = self.map.write().unwrap();
        s.insert(String::from(key), StdGauge::new());
    }

    fn incr(&self, key: &str) -> future::FutureResult<(), ()> {
        let mut s = self.map.write().unwrap();
        let e = s.entry(String::from(key)).or_insert(StdGauge::new());
        e.inc();
        future::ok(())
    }

    fn decr(&self, key: &str) -> future::FutureResult<(), ()> {
        let mut s = self.map.write().unwrap();
        let e = s.entry(String::from(key)).or_insert(StdGauge::new());
        e.dec();
        future::ok(())
    }

    fn status(&self, key: Option<&str>) {
        let s = self.map.read().unwrap();
        match key {
            Some(x) => println!("{:?}", s.get(x).unwrap().snapshot()),
            None => {
                for (k, v) in s.iter() {
                    println!("{}, {}", k, v.snapshot())
                }
            }
        }
    }

    fn new() -> Arc<Self> {
        Arc::new(Registry { map: RwLock::new(HashMap::new()) })
    }
}

struct Server {
    thread_pool: CpuPool,
    metrics: Arc<Registry>
}

#[derive(RustcEncodable)]
struct Message {
    ty: i32,
}

fn async_slow() -> future::FutureResult<i32, ()> {
    let mut rng = rand::thread_rng();
    let mut x = 0;
    for _ in 1..200000000 {
        x += rng.gen::<i32>();
    }
    future::ok(x)
}

fn async_fast() -> future::FutureResult<i32, ()> {
    future::ok(0)
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = BoxFuture<Response, io::Error>;

    fn call(&self, req: Request) -> Self::Future {
        //        let counter_instance = self.reporter.lock();
        let metrics = self.metrics.clone();
        let th = self.thread_pool.clone();
        self.thread_pool.spawn_fn(move || {
            let ty;

            let _ = metrics.incr("open");

            if req.path() == "/slow" {
                ty = th.spawn_fn(|| async_slow());
                let _ = metrics.incr("slow");
            } else {
                ty = th.spawn_fn(|| async_fast());
                let _ = metrics.incr("fast");
            }
            let _ = metrics.decr("open");

            metrics.status(None);

            ty
        }).then(|x| {
            let msg = Message { ty: x.unwrap() };
            let json = rustc_serialize::json::encode(&msg).unwrap();
            let mut response = Response::new();
            response.header("Content-Type", "application/json");
            response.body(&json);
            future::ok(response)
        }).boxed()
    }
}

fn main() {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let thread_pool = CpuPool::new(10);
    let registry = Registry::new();
    registry.add("open");
    registry.add("slow");
    registry.add("fast");

    TcpServer::new(tokio_minihttp::Http, addr).serve(move || {
        Ok(Server {
            thread_pool: thread_pool.clone(),
            metrics: registry.clone()
        })
    })
}