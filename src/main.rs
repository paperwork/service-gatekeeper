#[macro_use]
extern crate serde_derive;
extern crate frank_jwt;
use frank_jwt::{Algorithm, decode};
use hyper::server::conn::AddrStream;
use hyper::header::{HeaderMap, HeaderValue};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{service_fn, make_service_fn};
use futures::future::{self, Future};
// use serde::{Deserialize, Serialize};
use serde_json::json;
// use serde_json::Result;
use std::env;
use std::time::SystemTime;

type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    port: u16,
    jwt_secret: String,
    services: Vec<Service>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Service {
    name: String,
    target: String,
    endpoints: Vec<String>,
}

fn not_found(req: Request<Body>) -> BoxFut {
    println!("Not found: {:?}", req);
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NOT_FOUND;
    Box::new(future::ok(response))
}

fn unauthorized(req: Request<Body>) -> BoxFut {
    println!("Unauthorized: {:?}", req);
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::UNAUTHORIZED;
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
    let config_json: String = env::var("CONFIG_JSON").unwrap_or_else(|_| panic!("Please set CONFIG_JSON to the JSON configuration you need!"));
    let config: Config = serde_json::from_str(&config_json).unwrap_or_else(|_| panic!("CONFIG_JSON could not be parsed. Please check for errors!"));
    let addr = ([0, 0, 0, 0], config.port).into();

    let make_svc = make_service_fn(move |socket: &AddrStream| {
        let cloned_config = config.clone();
        let remote_addr = socket.remote_addr();
        service_fn(move |mut req: Request<Body>| {
            let headers_map = req.headers();
            if headers_map.contains_key("authorization") {
                let bearer_string = headers_map["authorization"].to_str().unwrap();
                let bearer:Vec<_> = bearer_string.split(' ').collect();
                let jwt = bearer[1].to_string();
                let payload = match decode(&jwt, &cloned_config.jwt_secret, Algorithm::HS512) {
                    Ok((_header, payload)) => payload,
                    Err(_e) => return unauthorized(req),
                };

                let exp = payload["exp"].as_u64().unwrap();
                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                if now.as_secs() > exp {
                    return unauthorized(req)
                }

                let json_string = json!(payload).to_string();
                *req.headers_mut() = resolve_authorization_header(req.headers(), &json_string);
            }

            for service in &cloned_config.services {
                for endpoint in &service.endpoints {
                    if req.uri().path().starts_with(endpoint) {
                        return hyper_reverse_proxy::call(remote_addr.ip(), &service.target, req);
                    }
                }
            }

            not_found(req)
        })
    });

    let server = Server::bind(&addr).serve(make_svc).map_err(|e| eprintln!("Server error: {}", e));
    println!("Running server on {:?}", addr);
    hyper::rt::run(server);
}
