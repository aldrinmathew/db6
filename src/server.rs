use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str,
};

use crate::{cli, http, json::Json};

pub fn listen(cl: &cli::Cli) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:".to_string() + &cl.port.to_string())?;
    println!("Got listener");
    loop {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                let res = handle_request(&mut stream);
                if res.is_err() {
                    eprintln!(
                        "Handling of request from {} failed with error: {}",
                        addr,
                        res.err().unwrap()
                    );
                }
            }
            Err(err) => {
                eprintln!("Error while handling incoming request: {}", err);
            }
        }
    }
}

pub fn handle_request(stream: &mut TcpStream) -> Result<(), String> {
    let header_end = "\r\n\r\n";
    let mut buf = Vec::<u8>::new();
    let mut req_complete = false;
    let mut temp_buff = [0; 512];
    let mut content_index: usize = 0;
    let mut reading_content = false;
    let mut pending_bytes = 0usize;
    let mut req: Option<http::Request> = None;
    while !req_complete {
        match stream.read(&mut temp_buff) {
            Ok(bytes_read) if bytes_read > 0 => {
                buf.extend_from_slice(&temp_buff[..bytes_read]);
                match str::from_utf8(&temp_buff) {
                    Ok(temp_str) => {
                        if !reading_content && temp_str.contains(header_end) {
                            let end_index = temp_str.find(header_end).unwrap();
                            let header_end_index = buf.len() - (bytes_read - end_index);
                            content_index = header_end_index + header_end.len();
                            match http::Request::from_bytes(&buf[..header_end_index]) {
                                Ok(head) => {
                                    if head.content_length.is_some() {
                                        pending_bytes = head.content_length.unwrap();
                                        pending_bytes -= bytes_read - end_index - header_end.len();
                                        reading_content = true;
                                    }
                                    req = Some(head);
                                }
                                Err(err) => {
                                    return Err(err);
                                }
                            }
                        } else if reading_content && pending_bytes > 0 {
                            if bytes_read < pending_bytes {
                                pending_bytes -= bytes_read
                            } else {
                                pending_bytes = 0;
                            }
                        }
                        if pending_bytes == 0 {
                            reading_content = false;
                            req_complete = true;
                        }
                    }
                    Err(err) => {
                        return Err(format!(
                            "Error while converting the request content to a string slice: {}",
                            err.to_string()
                        ));
                    }
                }
            }
            Ok(_) => {
                return Err("Client disconnected".to_string());
            }
            Err(err) => {
                return Err(err.to_string());
            }
        }
    }
    Json::parse(buf.as_slice());
    match req {
        Some(request) => {
            println!("Handling request to {}", request.route);
            let resp = "{ \"status\" : \"success\" }";
            let resp_str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: "
                .to_owned()
                + &resp.len().to_string()
                + "\r\n\r\n"
                + resp;
            match stream.write_all(resp_str.as_bytes()) {
                Ok(_) => match stream.flush() {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err.to_string()),
                },
                Err(err) => Err(err.to_string()),
            }
        }
        None => Err("Failed to parse header data from the request".to_string()),
    }
}
