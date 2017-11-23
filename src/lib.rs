extern crate hyper;
extern crate route_recognizer;
extern crate futures;

use std::str::FromStr;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use futures::future::{Future, ok};

use hyper::{Uri, Method, Headers, HttpVersion, Body, StatusCode, Error};
use hyper::server::Service;
use hyper::server::Request as HyperRequest;
use hyper::server::Response;

use route_recognizer::Router as RecognizerRouter;
use route_recognizer::Params;

pub struct Request {
	inner: HyperRequest,
	params: Params,
}

impl Deref for Request {
	type Target = HyperRequest;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Request {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl Request {
	fn new(hyper_req: HyperRequest, params: Params) -> Request {
		Request {
			inner: hyper_req,
			params: params,
		}
	}

	pub fn deconstruct(self) -> (Method, Uri, HttpVersion, Headers, Body) {
		self.inner.deconstruct()
	}

	pub fn body(self) -> Body {
		self.inner.body()
	}

	pub fn params(&self) -> &Params {
		&self.params
	}
}

pub struct Router {
	router: RecognizerRouter<Box<Service<Request = Request, Response = Response, Error = Error, Future = Box<Future<Item = Response, Error = Error>>>>>,
	subrouters: HashMap<String, Router>,
}

impl Router {
	pub fn new() -> Router {
		Router {
			router: RecognizerRouter::new(),
			subrouters: HashMap::new(),
		}
	}

	pub fn add<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = Request, Response = Response, Error = Error, Future = Box<Future<Item = Response, Error = Error>>>
	{
		self.router.add(route, Box::new(service));
	}

	pub fn add_router(&mut self, route: &str, subrouter: Router)
	{
		self.subrouters.insert(route.to_owned(), subrouter);
	}
}

impl Service for Router {
	type Request = HyperRequest;
	type Response = Response;
	type Error = Error;
	type Future = Box<Future<Item = Response, Error = Error>>;

	fn call(&self, mut req: HyperRequest) -> Self::Future {
		match self.router.recognize(req.path()) {
			Ok(service) => service.handler.call(Request::new(req, service.params)),
			Err(_) => {
				let path = req.uri().path().to_owned();
				let mut found = None;
				for prefix in self.subrouters.keys() {
					if path.starts_with(prefix) {
						// stop on first find.
						found = Some(prefix);
						break;
					}
				}

				match found {
					Some(found) => {

						let new_path = match path.trim_left_matches(found) {
							"" => "/",
							path => path
						};

						let new_uri = match Uri::from_str(&new_path) {
							Ok(uri) => uri,
							Err(e) => panic!("{:?}", e),
						};

						req.set_uri(new_uri);

						let subrouter = &self.subrouters[found];
						subrouter.call(req)
					}
					None => Box::new(ok(Response::new().with_status(StatusCode::NotFound)))
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
	}
}
