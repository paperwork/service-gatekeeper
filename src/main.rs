#[macro_use]
extern crate lazy_static;
extern crate frank_jwt;
use frank_jwt::{Algorithm, decode};
use hyper::server::conn::AddrStream;
use hyper::header::{HeaderMap, HeaderValue};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{service_fn, make_service_fn};
use futures::future::{self, Future};
use serde_json::json;
use std::env;

type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

lazy_static! {
    static ref PORT: String = env::var("PORT").unwrap_or_else(|_| panic!("Please set PORT to the port the gatekeeper should be running on!"));
    static ref JWT_SECRET: String = env::var("JWT_SECRET").unwrap_or_else(|_| panic!("Please set JWT_SECRET to the secret service-users uses to generate JWT tokens!"));
    static ref SERVICE_USERS: String = env::var("SERVICE_USERS").unwrap_or_else(|_| panic!("Please set SERVICE_USERS to the URL of service-users, e.g. http://service-users:8880"));
}

fn not_found(req: Request<Body>) -> BoxFut {
    println!("{:?}", req);
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NOT_FOUND;
    Box::new(future::ok(response))
}

fn resolve_authorization_header(headers: &HeaderMap<HeaderValue>, resolved_authorization: &str) -> HeaderMap<HeaderValue> {
    let mut result = HeaderMap::new();
    for (k, v) in headers.iter() {
        if k.as_str() == "authorization" {
            result.insert(k.clone(), HeaderValue::from_str(resolved_authorization).unwrap());
        } else {
            result.insert(k.clone(), v.clone());
        }
    }
    result
}

fn main() {
    let server_port: u16 = PORT.parse().unwrap();
    let addr = ([0, 0, 0, 0], server_port).into();

    let make_svc = make_service_fn(|socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        service_fn(move |mut req: Request<Body>| {
            let headers_map = req.headers();
            if headers_map.contains_key("authorization") {
                let bearer_string = headers_map["authorization"].to_str().unwrap();
                let bearer:Vec<_> = bearer_string.split(' ').collect();
                let jwt = bearer[1].to_string();
                let (_, payload) = decode(&jwt, &(*JWT_SECRET), Algorithm::HS512).unwrap();
                let json_string = json!(payload).to_string();
                *req.headers_mut() = resolve_authorization_header(req.headers(), &json_string);
            }

            if req.uri().path().starts_with("/users") {
                return hyper_reverse_proxy::call(remote_addr.ip(), &SERVICE_USERS, req)
            } else {
                not_found(req)
            }
        })
    });

    let server = Server::bind(&addr).serve(make_svc).map_err(|e| eprintln!("Server error: {}", e));
    println!("Running server on {:?}", addr);
    hyper::rt::run(server);
}
