extern crate hyper;
extern crate http;
extern crate futures;
extern crate hyper_async_router;
extern crate tokio_service;

use std::marker::PhantomData;

use futures::future::Future;
use futures::{IntoFuture};

use hyper::Body;
use hyper::server::{Http, Service};

use http::Request as HttpRequest;
use http::Response as HttpResponse;

use hyper_async_router::{Router, Params};

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

fn index(_: HttpRequest<Body>) -> Box<Future<Item = HttpResponse<Body>, Error = hyper::Error>>
{
    let body = Body::from("index".to_string());
    let mut response = HttpResponse::new(body);
    *response.status_mut() = ::http::StatusCode::OK;
    Box::new(futures::future::ok(response))
}

fn users(_: HttpRequest<Body>) -> Box<Future<Item = HttpResponse<Body>, Error = hyper::Error>>
{
    let body = Body::from("users".to_string());
    let mut response = HttpResponse::new(body);
    *response.status_mut() = ::http::StatusCode::OK;
    Box::new(futures::future::ok(response))
}

fn user(req: HttpRequest<Body>) -> Box<Future<Item = HttpResponse<Body>, Error = hyper::Error>>
{
    let params = req.extensions().get::<Params>().unwrap();
    let body = Body::from(format!("user: {:?}", params));
    let mut response = HttpResponse::new(body);
    *response.status_mut() = ::http::StatusCode::OK;
    Box::new(futures::future::ok(response))
}

fn sub(_: HttpRequest<Body>) -> Box<Future<Item = HttpResponse<Body>, Error = hyper::Error>>
{
    let body = Body::from("sub".to_string());
    let mut response = HttpResponse::new(body);
    *response.status_mut() = ::http::StatusCode::OK;
    Box::new(futures::future::ok(response))
}

fn sub_with_params(req: HttpRequest<Body>) -> Box<Future<Item = HttpResponse<Body>, Error = hyper::Error>>
{
    let params = req.extensions().get::<Params>().unwrap();
    let body = Body::from(format!("sub_with_params: {:?}", params));
    let mut response = HttpResponse::new(body);
    *response.status_mut() = ::http::StatusCode::OK;
    Box::new(futures::future::ok(response))
}

fn main() {
    let addr = "127.0.0.1:1337".parse().unwrap();

    let server = Http::new().bind(&addr, || {
        let mut subrouter = Router::new();
        subrouter.add("/", service_fn(sub));
        subrouter.add("/:id/foobar/:bar", service_fn(sub_with_params));

        let mut router = Router::new();
        router.add("/", service_fn(index));
        router.add("/users", service_fn(users));
        router.add("/users/:id", service_fn(user));
        router.add_router("/subrouter", subrouter);
        Ok(router)}).unwrap();
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
}