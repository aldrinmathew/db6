use std::str;

#[derive(Clone)]
pub enum HttpMethod {
    GET,
    POST,
    DELETE,
    UPDATE,
}

pub enum ContentType {
    TextPlain,
    ApplicationJson,
    None,
}

pub struct Headers {
    pub method: HttpMethod,
    pub route: String,
    pub http_version: String,
    pub host: String,
    pub content_type: ContentType,
    pub content_length: usize,
}

pub struct Request {
    pub header: Headers,
    pub content: Vec<u8>,
}

impl Headers {
    pub fn from_bytes(bytes: &[u8]) -> Result<Headers, String> {
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
                    method = match first_header[0] {
                        "GET" => Some(HttpMethod::GET),
                        "POST" => Some(HttpMethod::POST),
                        "UPDATE" => Some(HttpMethod::UPDATE),
                        "DELETE" => Some(HttpMethod::DELETE),
                        _ => {
                            return Err("Invalid HTTP method found in request".to_string());
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
                                    content_type = match value {
                                        "text/plain" => Some(ContentType::TextPlain),
                                        "application/json" => Some(ContentType::ApplicationJson),
                                        _ if match method.clone().unwrap() {
                                            HttpMethod::POST => true,
                                            HttpMethod::UPDATE => true,
                                            _ => false,
                                        } =>
                                        {
                                            Some(ContentType::None)
                                        }
                                        _ => None,
                                    };
                                }
                                "Content-Length" => {
                                    content_length = match value.parse::<usize>() {
                                        Ok(len) => Some(len),
                                        Err(_) => None,
                                    };
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
                if content_type.is_none() {
                    return Err("Invalid request - Content type is not found".to_string());
                }
                if content_length.is_none() {
                    return Err("Invalid request - Content length is not found".to_string());
                }
                Ok(Headers {
                    method: method.unwrap(),
                    route: route.unwrap(),
                    http_version: http_version.unwrap(),
                    host: host.unwrap(),
                    content_type: content_type.unwrap(),
                    content_length: content_length.unwrap(),
                })
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
