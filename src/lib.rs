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

enum MethodChecker {
	All,
	AnyOf(Vec<::http::Method>),
}

struct Handler{
	service: Box<Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>>,
	methods: MethodChecker,
}

pub struct Router {
	router: RecognizerRouter<Handler>,
	subrouters: HashMap<String, Router>,
}

impl Router {
	pub fn new() -> Router {
		Router {
			router: RecognizerRouter::new(),
			subrouters: HashMap::new(),
		}
	}

	pub fn any<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.router.add(route, Handler{ service: Box::new(service), methods: MethodChecker::All});
	}

	pub fn get<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::GET]);
	}

	pub fn post<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::POST]);
	}

	pub fn put<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::PUT]);
	}

	pub fn options<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::OPTIONS]);
	}

	pub fn head<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::HEAD]);
	}

	pub fn delete<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::DELETE]);
	}

	pub fn connect<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::CONNECT]);
	}

	pub fn patch<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::PATCH]);
	}

	pub fn trace<T>(&mut self, route: &str, service: T)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.add_with_methods(route, service, vec![::http::Method::TRACE]);
	}

	pub fn add_with_methods<T>(&mut self, route: &str, service: T, methods: Vec<::http::Method>)
		where T: 'static + Service<Request = HttpRequest<Body>, Response = HttpResponse<Body>, Error = Error, Future = Box<Future<Item = HttpResponse<Body>, Error = Error>>>
	{
		self.router.add(route, Handler{ service: Box::new(service), methods: MethodChecker::AnyOf(methods)});
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
			Ok(matched) => {
				let mut request: HttpRequest<Body> = HttpRequest::from(req);
				use MethodChecker::*;
				let matched_method = match matched.handler.methods {
					All => true,
					AnyOf(ref methods) => methods.contains(request.method())
				};

				if matched_method {
					request.extensions_mut().insert(matched.params);
					Box::new(matched.handler.service.call(request).map(|r| Response::from(r)))
				} else {
					Box::new(ok(Response::new().with_status(StatusCode::MethodNotAllowed)))
				}

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
