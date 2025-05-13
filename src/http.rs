use std::{
    fmt::Display,
    str::{self, FromStr},
};

use crate::json::Json;

#[derive(Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    TRACE,
    CONNECT,
}

impl HttpMethod {
    pub fn supports_request_body(&self) -> bool {
        use HttpMethod::*;
        match self {
            POST | PUT | PATCH | DELETE | OPTIONS => true,
            _ => false,
        }
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "PATCH" => Ok(HttpMethod::PATCH),
            "DELETE" => Ok(HttpMethod::DELETE),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "TRACE" => Ok(HttpMethod::TRACE),
            "CONNECT" => Ok(HttpMethod::CONNECT),
            _ => Err("Invalid HTTP Method".to_string()),
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::CONNECT => "CONNECT",
        })
    }
}

pub enum ContentType {
    TextPlain,
    ApplicationJson,
    ApplicationOctetStream,
    None,
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text/plain" => Ok(ContentType::TextPlain),
            "application/json" => Ok(ContentType::ApplicationJson),
            "application/octet-stream" => Ok(ContentType::ApplicationOctetStream),
            _ => Err("Invalid content type".to_string()),
        }
    }
}

pub enum Body {
    TextPlain(String),
    ApplicationJson(Json),
    ApplicationOctetStream(Vec<u8>),
    None,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            ContentType::TextPlain => "text/plain",
            ContentType::ApplicationJson => "application/json",
            ContentType::ApplicationOctetStream => "application/octet-stream",
            ContentType::None => "",
        })
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Body::TextPlain(string) => string.fmt(f),
            Body::ApplicationJson(json) => json.fmt(f),
            Body::ApplicationOctetStream(vec) => vec
                .iter()
                .map(|byte| format!("{:02X}", byte))
                .collect::<Vec<_>>()
                .join(" ")
                .fmt(f),
            Body::None => Ok(()),
        }
    }
}

pub struct Request {
    pub method: HttpMethod,
    pub route: String,
    pub http_version: String,
    pub host: String,
    pub content_type: Option<ContentType>,
    pub content_length: Option<usize>,
    pub content: Vec<u8>,
}

impl Request {
    pub fn from_bytes(bytes: &[u8]) -> Result<Request, String> {
        match str::from_utf8(bytes) {
            Ok(header) => {
                let mut method: Option<HttpMethod> = None;
                let mut route: Option<String> = None;
                let mut http_version: Option<String> = None;
                let mut host: Option<String> = None;
                let mut content_type: Option<ContentType> = None;
                let mut content_length: Option<usize> = None;
                let headers: Vec<&str> = header.split("\r\n").collect();
                if headers.len() > 1 {
                    let first_header: Vec<&str> = headers[0].split(" ").collect();
                    if first_header.len() != 3 {
                        return Err(
                            "The first header of this request is of invalid format".to_string()
                        );
                    }
                    method = match first_header[0].parse::<HttpMethod>() {
                        Ok(method_val) => Some(method_val),
                        Err(err) => {
                            return Err(err);
                        }
                    };
                    route = Some(first_header[1].to_string());
                    http_version = Some(first_header[2].to_string());
                    for line in headers[1..].iter() {
                        if line.contains(": ") {
                            let colon = line.find(": ").unwrap();
                            if colon + 2 == line.len() {
                                continue;
                            }
                            let name = &line[..colon];
                            let value = &line[(colon + 2)..];
                            match name {
                                "Host" => {
                                    host = Some(value.to_string());
                                }
                                "Content-Type" => {
                                    if method.clone().unwrap().supports_request_body() {
                                        content_type = match value.parse::<ContentType>() {
                                            Ok(cont_ty) => Some(cont_ty),
                                            Err(err) => {
                                                return Err(err);
                                            }
                                        };
                                    }
                                }
                                "Content-Length" => {
                                    if method.clone().unwrap().supports_request_body() {
                                        content_length = match value.parse::<usize>() {
                                            Ok(len) => Some(len),
                                            Err(_) => None,
                                        };
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                if method.is_none() {
                    return Err("Invalid request - Method was not found".to_string());
                }
                if route.is_none() {
                    return Err("Invalid request - Route was not found".to_string());
                }
                if http_version.is_none() {
                    return Err("Invalid request - HTTP version is not found".to_string());
                }
                if host.is_none() {
                    return Err("Invalid request - Host is not found".to_string());
                }
                let mut content = Vec::<u8>::new();
                if content_length.is_some() {
                    content.reserve_exact(content_length.unwrap());
                }
                Ok(Request {
                    method: method.unwrap(),
                    route: route.unwrap(),
                    http_version: http_version.unwrap(),
                    host: host.unwrap(),
                    content_type: content_type,
                    content_length: content_length,
                    content,
                })
            }
            Err(err) => Err(err.to_string()),
        }
    }

    fn parse_content(&mut self, bytes: Vec<u8>) {}
}
