use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    path::Path,
};

use chrono::Local;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Failed to establish a connection: {}", e);
            }
        }
    }
}

fn get_formated_datetime() -> String {
    let now = Local::now();
    now.format("%Y.%m.%d %H:%M").to_string()
}

fn parse_form_data(body: &str) -> HashMap<String, String> {
    let mut data = HashMap::new();
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_value = urlencoding::decode(value)
                .unwrap_or_else(|_| value.into())
                .to_string();
            data.insert(key.to_string(), decoded_value);
        }
    }
    data
}

fn handle_post_request(mut stream: TcpStream, headers: HashMap<String, String>, body: String) {
    println!("Handling POST request for /submit-demo-request");
    let form_data = parse_form_data(&body);

    let name = form_data.get("name").map_or("", |s| s.as_str());
    let email = form_data.get("email").map_or("", |s| s.as_str());
    let phone = form_data.get("phone").map_or("", |s| s.as_str());
    let comments = form_data.get("comments").map_or("", |s| s.as_str());
    let timestamp = get_formated_datetime();

    let csv_file_path = "demo_requests.csv";
    let file_exists = Path::new(csv_file_path).exists();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(csv_file_path)
        .unwrap();

    if !file_exists {
        if let Err(e) = writeln!(file, "Timestamp,Name,Email,Phone,Comments") {
            eprintln!("Couldn't write header to CSV: {}", e);
        }
    }

    let record = format!(
        "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
        timestamp.replace('"', "\"\""),
        name.replace('"', "\"\""),
        email.replace('"', "\"\""),
        phone.replace('"', "\"\""),
        comments.replace('"', "\"\"")
    );

    if let Err(e) = writeln!(file, "{}", record) {
        eprintln!("Couldn't write to CSV: {}", e);
        let response =
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    println!("Successfully wrote to CSV: {}", record);
    let response_body = "Form submitted successfully!";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut request_line = String::new();
    if buf_reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        eprintln!("Failed to read request line or request was empty.");
        return;
    }

    let request_parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if request_parts.len() < 2 {
        eprintln!("Malformed request line: {}", request_line);
        return;
    }
    let method = request_parts[0];
    let path_from_request = request_parts[1];

    println!("Request: {}", request_line.trim());

    let mut headers = HashMap::new();
    let mut content_length = 0;
    loop {
        let mut header_line = String::new();
        if buf_reader.read_line(&mut header_line).is_err() || header_line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = header_line.trim().split_once(": ") {
            headers.insert(name.to_lowercase().to_string(), value.to_string());
            if name.to_lowercase() == "content-length" {
                content_length = value.parse::<usize>().unwrap_or(0);
            }
        }
    }

    if method == "POST" && path_from_request == "/submit-demo-request" {
        let mut body_bytes = vec![0; content_length];
        if content_length > 0 {
            if buf_reader.read_exact(&mut body_bytes).is_err() {
                eprintln!("Failed to read request body fully.");
                let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 19\r\nConnection: close\r\n\r\nError reading body";
                stream
                    .write_all(response.as_bytes())
                    .unwrap_or_else(|e| eprintln!("Error sending 400 response: {}", e));
                return;
            }
        }
        let body_string = String::from_utf8_lossy(&body_bytes).to_string();
        handle_post_request(stream, headers, body_string);
        return;
    }

    let mut status_line_to_send: &str;
    let mut file_path_to_serve: String;
    let mut content_type_to_send: &str;
    let mut serve_binary_content = false;

    if path_from_request == "/" {
        status_line_to_send = "HTTP/1.1 200 OK";
        file_path_to_serve = "index.html".to_string();
        content_type_to_send = "text/html; charset=utf-8";
    } else {
        let requested_relative_path = path_from_request.trim_start_matches('/').to_string();

        if Path::new(&requested_relative_path).exists() {
            status_line_to_send = "HTTP/1.1 200 OK";
            file_path_to_serve = requested_relative_path.clone();

            if requested_relative_path.ends_with(".css") {
                content_type_to_send = "text/css; charset=utf-8";
            } else if requested_relative_path.ends_with(".html") {
                content_type_to_send = "text/html; charset=utf-8";
            } else if requested_relative_path.ends_with(".js") {
                content_type_to_send = "application/javascript; charset=utf-8";
            } else if requested_relative_path.ends_with(".jpg")
                || requested_relative_path.ends_with(".jpeg")
            {
                content_type_to_send = "image/jpeg";
                serve_binary_content = true;
            } else if requested_relative_path.ends_with(".png") {
                content_type_to_send = "image/png";
                serve_binary_content = true;
            } else if requested_relative_path.ends_with(".ico") {
                content_type_to_send = "image/x-icon";
                serve_binary_content = true;
            } else {
                content_type_to_send = "application/octet-stream";
                serve_binary_content = true;
            }
        } else {
            println!(
                "File '{}' not found. Targeting 404.html.",
                requested_relative_path
            );
            status_line_to_send = "HTTP/1.1 404 NOT FOUND";
            file_path_to_serve = "404.html".to_string();
            content_type_to_send = "text/html; charset=utf-8";
            serve_binary_content = false;
        }
    }

    if serve_binary_content {
        match fs::read(&file_path_to_serve) {
            Ok(binary_contents) => {
                let response = format!(
                    "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
                    status_line_to_send,
                    binary_contents.len(),
                    content_type_to_send
                );
                stream
                    .write_all(response.as_bytes())
                    .unwrap_or_else(|e| eprintln!("Error writing headers for binary: {}", e));
                stream
                    .write_all(&binary_contents)
                    .unwrap_or_else(|e| eprintln!("Error writing binary content: {}", e));
                println!("Successfully sent binary file '{}'.", file_path_to_serve);
            }
            Err(e_read) => {
                eprintln!(
                    "Error reading binary file '{}': {}. Sending 404.",
                    file_path_to_serve, e_read
                );
                let status_404 = "HTTP/1.1 404 NOT FOUND";
                let body_404 = "404 Not Found";
                let response_404 = format!(
                    "{}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
                    status_404, body_404.len(), body_404
                );
                stream
                    .write_all(response_404.as_bytes())
                    .unwrap_or_else(|e| eprintln!("Error writing 404 for binary fail: {}", e));
            }
        }
    } else {
        let response_contents_string = match fs::read_to_string(&file_path_to_serve) {
            Ok(contents) => {
                println!("Successfully read text file '{}'.", file_path_to_serve);
                contents
            }
            Err(e_read) => {
                eprintln!(
                    "Error reading text file '{}': {}.",
                    file_path_to_serve, e_read
                );
                if file_path_to_serve != "404.html" {
                    status_line_to_send = "HTTP/1.1 404 NOT FOUND";
                    content_type_to_send = "text/html; charset=utf-8";
                    match fs::read_to_string("404.html") {
                        Ok(c404) => {
                            println!("Successfully read fallback '404.html'.");
                            c404
                        }
                        Err(e_404_read) => {
                            eprintln!(
                                "Error reading fallback '404.html': {}. Sending hardcoded 404.",
                                e_404_read
                            );
                            "<html><head><title>404 Not Found</title></head><body><h1>404 Not Found</h1><p>The requested resource was not found, and the 404.html error page is also missing or unreadable.</p></body></html>".to_string()
                        }
                    }
                } else {
                    eprintln!("'404.html' (targeted as error page) failed to read. Sending hardcoded 404.");
                    "<html><head><title>404 Not Found</title></head><body><h1>404 Not Found</h1><p>The 404.html error page is missing or unreadable.</p></body></html>".to_string()
                }
            }
        };

        let length = response_contents_string.len();
        let response = format!(
            "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n{}",
            status_line_to_send, length, content_type_to_send, response_contents_string
        );
        stream
            .write_all(response.as_bytes())
            .unwrap_or_else(|e| eprintln!("Error writing text response: {}", e));
    }

    match stream.flush() {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to flush stream: {}", e),
    }
    println!("Response processing complete for '{}'", path_from_request);
}
