# hyper-async-router

Simple router for async [hyper](https://hyper.rs/) implemented as a wrapper around [router-recognizer](https://github.com/conduit-rust/route-recognizer.rs).

The basics, see [examples/server.rs](https://github.com/havarnov/hyper-async-router/blob/master/examples/server.rs) for more info:
```rust

// must impl hyper::server::Service in some way or another
fn admin_user_post(req: RouterRequest) -> Box<Future<Item = Response, Error = hyper::Error>> {
    let params = format!("{:?}", req.params());
    Box::new(futures::future::ok(
        Response::new()
            .with_header(ContentLength(params.len() as u64))
            .with_body(params)))
}

fn main() {
    let addr = "127.0.0.1:1337".parse().unwrap();

    let server = Http::new().bind(&addr, || {
        let mut admin_router = Router::new();
        subrouter.add("/", admin_index);
        subrouter.add("/users/:id/posts/:post_id", admin_user_post);
        
        let mut router = Router::new();
        router.add("/", index);
        router.add("/users", users);
        router.add_router("/admin", admin_router);
        
        Ok(router)}).unwrap();
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
}
```

## build

```sh
# normal build
cargo build

# run example server
cargo run --example server
```
