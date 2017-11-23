extern crate hyper;
extern crate route_recognizer;
extern crate futures;

use std::str::FromStr;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use futures::future::Future;

use hyper::StatusCode;
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

	pub fn deconstruct(self) -> (::hyper::Method, ::hyper::Uri, ::hyper::HttpVersion, ::hyper::Headers, ::hyper::Body) {
		self.inner.deconstruct()
	}

	pub fn body(self) -> ::hyper::Body {
		self.inner.body()
	}

	pub fn params(&self) -> &Params {
		&self.params
	}
}

pub struct Router {
	router: RecognizerRouter<Box<Service<Request = Request, Response = Response, Error = hyper::Error, Future = Box<Future<Item = Response, Error = hyper::Error>>>>>,
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
		where T: 'static + Service<Request = Request, Response = Response, Error = hyper::Error, Future = Box<Future<Item = Response, Error = hyper::Error>>>
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
	type Error = hyper::Error;
	type Future = Box<Future<Item = Response, Error = hyper::Error>>;

	fn call(&self, mut req: HyperRequest) -> Self::Future {
		match self.router.recognize(req.path()) {
			Ok(service) => service.handler.call(Request::new(req, service.params)),
			Err(_) => {
				let u = req.uri().to_owned();
				let p = u.path().to_owned();
				let mut found = None;
				for prefix in self.subrouters.keys() {
					if p.starts_with(prefix) {
						found = Some(prefix);
						println!("{:?}", found);
					}
				}

				match found {
					Some(found) => {
						let new_path = match p.trim_left_matches(found) {
							"" => "/",
							p => p
						};
						let uu = ::hyper::Uri::from_str(&new_path);
						let uuu = match uu {
							Ok(uu) => uu,
							Err(e) => panic!("{:?}", e),
						};
						println!("{:?}", found);
						req.set_uri(uuu);
						let s = &self.subrouters[found];
						s.call(req)
					}
					None => {
						Box::new(futures::future::ok(
						Response::new()
							.with_status(StatusCode::NotFound)))
					}
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
