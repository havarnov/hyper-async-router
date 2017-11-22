extern crate hyper;
extern crate route_recognizer;
extern crate futures;
extern crate hyper_async_router;
extern crate tokio_service;

use std::marker::PhantomData;

// use futures::future::FutureResult;
use futures::future::Future;
use futures::{IntoFuture};

// use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

use hyper_async_router::Router;
use hyper_async_router::Request as RouterRequest;

/// A service implemented by a closure.
pub struct ServiceFn<F, R> {
    f: F,
    _ty: PhantomData<fn() -> R>, // don't impose Sync on R
}

impl<F, R, S> Service for ServiceFn<F, R>
    where F: Fn(R) -> S,
          S: IntoFuture,
{
    type Request = R;
    type Response = S::Item;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        (self.f)(req).into_future()
    }
}

/// Returns a `Service` backed by the given closure.
pub fn service_fn<F, R, S>(f: F) -> ServiceFn<F, R>
    where F: Fn(R) -> S,
          S: IntoFuture,
{
    ServiceFn {
        f: f,
        _ty: PhantomData,
    }
}

static TEXT: &'static str = "Hello, World!";

fn index(req: RouterRequest) -> Box<Future<Item = Response, Error = hyper::Error>> {
    Box::new(futures::future::ok(
        Response::new()
            .with_header(ContentLength(TEXT.len() as u64))
            .with_body(TEXT)))
}

fn index2(req: RouterRequest) -> Box<Future<Item = Response, Error = hyper::Error>> {
    Box::new(futures::future::ok(
        Response::new()
            .with_header(ContentLength("index2".len() as u64))
            .with_body("index2")))
}

fn index3(req: RouterRequest) -> Box<Future<Item = Response, Error = hyper::Error>> {
    Box::new(futures::future::ok(
        Response::new()
            .with_header(ContentLength("index3".len() as u64))
            .with_body("index3")))
}

fn index4(req: RouterRequest) -> Box<Future<Item = Response, Error = hyper::Error>> {
    let t = format!("index4: {:?}", req.params());
    Box::new(futures::future::ok(
        Response::new()
            .with_header(ContentLength(t.len() as u64))
            .with_body(t)))
}

fn user(req: RouterRequest) -> Box<Future<Item = Response, Error = hyper::Error>> {
    let msg = "HELLO USER!";
    Box::new(futures::future::ok(
        Response::new()
            .with_header(ContentLength(msg.len() as u64))
            .with_body(msg)))
}

fn main() {
    let addr = "127.0.0.1:1337".parse().unwrap();

    let server = Http::new().bind(&addr, || {
        let mut subrouter = Router::new();
        subrouter.add("/", service_fn(index2));
        subrouter.add("/ind3", service_fn(index3));
        subrouter.add("/ind3/:id/foobar/:rall", service_fn(index4));
        let mut router = Router::new();
        router.add("/", service_fn(index));
        router.add("/user", service_fn(user));
        router.add_router("/subrouter", subrouter);
        Ok(router)}).unwrap();
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
}