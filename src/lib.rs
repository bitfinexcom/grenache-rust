#[macro_use]
extern crate log;
extern crate hyper;
extern crate serde_json;
extern crate tokio;
extern crate uuid;

use hyper::header::HeaderValue;
use hyper::rt::Stream;
use hyper::Client;
use hyper::Request;
use hyper::{Body, Method};
use serde_json::Value;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::sync::mpsc;
use std::{thread, time};
use tokio::runtime::Runtime;
use uuid::Uuid;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl Error {
    pub fn new<T: Into<String>>(msg: T) -> Error {
        Error { msg: msg.into() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", &self.msg)
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

trait JsonPost {
    fn execute_post(url: &str, payload: &str) -> Result<Value> {
        let request = Self::create_json_post_request(url, payload)?;
        let response = Self::handle_request(request)?;
        let decoded_string = String::from_utf8(response.clone())?;
        match serde_json::from_str(&decoded_string) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::new(format!(
                "Could not parse \"{}\" as json: {}",
                decoded_string, e
            ))
            .into()),
        }
    }

    fn create_json_post_request(url: &str, payload: &str) -> Result<hyper::Request<hyper::Body>> {
        let uri: hyper::Uri = url.parse()?;
        let mut req = Request::new(Body::from(payload.to_string()));
        *req.method_mut() = Method::POST;
        *req.uri_mut() = uri.clone();
        req.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=UTF-8"),
        );
        return Ok(req);
    }

    fn handle_request(request: Request<hyper::Body>) -> Result<Vec<u8>> {
        let client = Client::new();
        let post = client.request(request);
        let mut tokio_runtime = Runtime::new()?;
        let future = tokio_runtime.block_on(post)?;
        let response_body = tokio_runtime.block_on(future.into_body().concat2())?;
        Ok(response_body.to_vec())
    }
}

pub trait Grenache {
    fn uuid() -> String {
        Uuid::new_v4().to_string()
    }

    fn create_announce_payload(service: &str, port: &str) -> String {
        format!(
            r#"{{"data": ["{}", {}], "rid": "{}"}}"#,
            service,
            port,
            Self::uuid()
        )
    }

    fn create_lookup_payload(service: &str) -> String {
        format!(r#"{{"data": "{}", "rid": "{}"}}"#, service, Self::uuid())
    }

    fn lookup(&self, service: &str) -> Result<Value>;

    fn announce(&self, service: &str, port: u16) -> Result<Value>;

    fn start_announcing(&mut self, service: &str, port: u16) -> Result<()>;

    fn stop_announcing(&mut self, service: &str) -> Result<()>;

    #[allow(unused_variables)]
    fn put(item: &str) -> Result<()> {
        unimplemented!();
    }

    #[allow(unused_variables)]
    fn get(key: &str) -> Result<()> {
        unimplemented!();
    }
}

#[derive(PartialEq)]
enum AnnounceMessage {
    STOP,
}

#[derive(Debug, Clone)]
pub struct GrenacheClient {
    url: String,
    senders: HashMap<String, mpsc::Sender<AnnounceMessage>>,
}

impl JsonPost for GrenacheClient {}

impl GrenacheClient {
    pub fn new<T: Into<String>>(url: T) -> GrenacheClient {
        GrenacheClient {
            url: url.into(),
            senders: HashMap::new(),
        }
    }

    fn execute_announce(url: &str, service: &str, port: u16) -> Result<Value> {
        let payload = Self::create_announce_payload(service, &port.to_string());
        Self::execute_post(&format!("{}/announce", url), &payload)
    }

    fn execute_lookup(url: &str, service: &str) -> Result<Value> {
        let payload = Self::create_lookup_payload(service);
        Self::execute_post(&format!("{}/lookup", url), &payload)
    }

    pub fn attempt_announce(url: &str, service: &str, port: u16, period: u64) -> Result<Value> {
        let mut result: Result<Value>;
        while {
            result = Self::execute_announce(url, service, port);
            result.is_err()
        } {
            thread::sleep(time::Duration::from_secs(period));
            error!(
                "Failed to announce {}. Retrying in {} seconds",
                service, period
            );
        }
        return result;
    }

    pub fn attempt_lookup(&self, service: &str) -> Result<Vec<String>> {
        let period = 1;
        let mut result: Result<Value>;
        while {
            result = Self::execute_lookup(&self.url, service);
            result.is_err() || Self::result_is_empty_array_or_error(&result)
        } {
            thread::sleep(time::Duration::from_secs(period));
            error!(
                "Failed to Lookup {}. Retrying in {} seconds",
                service, period
            );
        }
        if let Ok(json) = result {
            return Self::get_hosts_from_array(json);
        } else {
            return Err(Error::new("Failed to perform lookup").into());
        }
    }

    pub fn result_is_empty_array_or_error(result: &Result<serde_json::Value>) -> bool {
        if let Ok(json) = result {
            debug!("Parsed json is {}", json);
            if json.to_string() == "[]" {
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    fn get_hosts_from_array(json_from_lookup: serde_json::Value) -> Result<Vec<String>> {
        let mut vec_for_hosts: Vec<String> = Vec::new();
        if let Some(array) = json_from_lookup.as_array() {
            for json_address in array {
                if let Some(url_str) = json_address.as_str() {
                    let string = url_str.to_string();
                    if string.is_empty() || string == "[]" || string == "null" {
                        return Err(Error::new(format!(
                            "Invalid string type in response: {}",
                            string
                        ))
                        .into());
                    } else {
                        vec_for_hosts.push(string);
                    }
                } else {
                    return Err(Error::new(format!(
                        "Invalid string type in response: {}",
                        json_address
                    ))
                    .into());
                }
            }
        } else {
            return Err(Error::new(format!(
                "Expected json array, received: {}",
                json_from_lookup
            ))
            .into());
        }
        return Ok(vec_for_hosts);
    }
}

impl Grenache for GrenacheClient {
    fn announce(&self, service: &str, port: u16) -> Result<Value> {
        Self::execute_announce(&self.url, service, port)
    }

    fn lookup(&self, service: &str) -> Result<Value> {
        Self::execute_lookup(&self.url, service)
    }

    fn start_announcing(&mut self, service: &str, port: u16) -> Result<()> {
        let movable_service = String::from(service);
        let movable_url = self.url.clone();
        let (tx, rx) = mpsc::channel();
        if self.senders.contains_key(service) {
            return Err(Error::new(format!("Already announcing {}", service)).into());
        } else {
            self.senders.insert(service.to_string(), tx);
        }

        thread::spawn(move || {
            loop {
                if let Ok(msg) = rx.try_recv() {
                    if msg == AnnounceMessage::STOP {
                        return;
                    }
                } else {
                    match Self::attempt_announce(&movable_url, &movable_service, port, 1) {
                        Ok(_) => info!("Successfully announced {}", &movable_service),
                        Err(_) => error!("Failed to announce {}", &movable_service),
                    }
                    // peers are by default dropped after 2 minutes by grenache, so re-announce every minute
                    thread::sleep(time::Duration::from_secs(60));
                }
            }
        });
        Ok(())
    }

    fn stop_announcing(&mut self, service: &str) -> Result<()> {
        if let Some(tx) = self.senders.remove(service) {
            tx.send(AnnounceMessage::STOP)?;
            Ok(())
        } else {
            Err(Error::new(format!("Wasn't announcing {}", service)).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announce_then_lookup() {
        let service = "rest:net:util";
        let service_port = 31_337u16;
        let grape_ip = "127.0.0.1";
        let api_port = "30001";
        let client = GrenacheClient::new(format!("http://{}:{}", grape_ip, api_port));
        let rhs = client.lookup(service).unwrap();
        assert_eq!(Value::Null, rhs[0]);
        client.announce(service, service_port).unwrap();
        let lhs = format!("{}:{}", grape_ip, service_port);
        let rhs = client.lookup(service).unwrap();
        assert_eq!(lhs, rhs[0]);
    }
}
