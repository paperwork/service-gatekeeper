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
use std::collections::HashMap;
use std::pin::Pin;
use url::Url;

type BoxFut = Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>>;

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
        result.insert(k.clone(), v.clone());
    }

    result.insert("authorization", HeaderValue::from_str(resolved_authorization).unwrap());
    println!("resolve_authorization_header: {:?}", result);

    result
}

fn get_json_from_jwt(config: Config, jwt: String) -> Option<String> {
    let payload = match decode(&jwt, &config.jwt_secret, Algorithm::HS512) {
        Ok((_header, payload)) => payload,
        Err(err) => {
            println!("get_json_from_jwt: Error {:?}", err);
            return None;
        },
    };

    println!("get_json_from_jwt: Ok {:?}", payload);

    let exp = payload["exp"].as_u64().unwrap();
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    if now.as_secs() > exp {
        println!("get_json_from_jwt: JWT expired!");
        return None;
    }

    return Some(json!(payload).to_string());
}

fn get_access_token_from_headers(req: &Request<Body>) -> Option<String> {
    let headers_map = req.headers();

    println!("Checking headers for access token: {:?}", headers_map);

    if headers_map.contains_key("authorization") {
        println!("Found authorization key!");

        let bearer_string = headers_map["authorization"].to_str().unwrap();
        let bearer:Vec<_> = bearer_string.split(' ').collect();
        return Some(bearer[1].to_string());
    }

    return None;
}

fn get_access_token_from_params(req: &Request<Body>) -> Option<String> {
    let uri_string = req.uri().to_string();
    println!("{:?}", uri_string);
    let request_url = match Url::parse(&["http://localhost", &uri_string].concat()) {
        Ok(url) => url,
        Err(err) => {
            println!("{:?}", err);
            return None;
        },
    };

    let params: HashMap<_, _> = request_url.query_pairs().into_owned().collect();

    println!("Checking params for access token: {:?}", params);

    return match params.get("_accessToken") {
        Some(acc_tok) => Some(acc_tok.to_string()),
        None => None
    }
}

#[tokio::main]
async fn main() {
    let config_json: String = env::var("CONFIG_JSON").unwrap_or_else(|_| panic!("Please set CONFIG_JSON to the JSON configuration you need!"));
    let config: Config = serde_json::from_str(&config_json).unwrap_or_else(|_| panic!("CONFIG_JSON could not be parsed. Please check for errors!"));
    let addr = ([0, 0, 0, 0], config.port).into();

    let make_svc = make_service_fn(move |socket: &AddrStream| {
        let cloned_config = config.clone();
        let remote_addr = socket.remote_addr();
        service_fn(move |mut req: Request<Body>| {
            let access_token = match get_access_token_from_headers(&req) {
                Some(acc_tok) => Some(acc_tok),
                None => get_access_token_from_params(&req),
            };

            match access_token {
                Some(acc_tok) => {
                    println!("Access token: {:?}", acc_tok);
                    let json_string = match get_json_from_jwt(cloned_config.clone(), acc_tok) {
                        Some(json_from_jwt) => json_from_jwt,
                        None => return unauthorized(req),
                    };

                    *req.headers_mut() = resolve_authorization_header(req.headers(), &json_string);
                },
                _ => {
                    println!("Access token: -");
                }
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

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
