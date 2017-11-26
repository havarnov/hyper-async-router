extern crate hyper;
extern crate http;
extern crate route_recognizer;
extern crate futures;

use std::str::FromStr;
use std::collections::HashMap;

use futures::future::{Future, ok};

use hyper::{Uri, Body, StatusCode, Error};
use hyper::server::Service;
use hyper::server::Request as HyperRequest;
use hyper::server::Response;

use http::Request as HttpRequest;
use http::Response as HttpResponse;

use route_recognizer::Router as RecognizerRouter;
pub use route_recognizer::Params;

pub struct Router {
	router: RecognizerRouter<Box<Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>>>,
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
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
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
			Ok(service) => {
				let mut request: HttpRequest<Body> = HttpRequest::from(req);
				request.extensions_mut().insert(service.params);
				Box::new(service.handler.call(request).map(|r| Response::from(r)))
			},
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
						Box::new(subrouter.call(req).map(|r| Response::from(r)))
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
