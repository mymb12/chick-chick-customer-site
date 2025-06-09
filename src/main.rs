use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    path::Path,
};

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

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line_result = buf_reader.lines().next();

    if request_line_result.is_none() {
        eprintln!("Received an empty request.");
        // Optionally send a 400 Bad Request response
        return;
    }
    let request_line = match request_line_result.unwrap() {
        Ok(line) => line,
        Err(e) => {
            eprintln!("Error reading request line: {}", e);
            // Optionally send a 400 Bad Request response
            return;
        }
    };

    println!("Request: {}", request_line);

    let path_from_request = request_line.split_whitespace().nth(1).unwrap_or("/");

    let mut status_line_to_send: &str;
    let mut file_path_to_serve: String; // Changed to file_path_to_serve for clarity
    let mut content_type_to_send: &str;
    let mut serve_binary_content = false; // Flag for binary files like images

    if path_from_request == "/" {
        status_line_to_send = "HTTP/1.1 200 OK";
        file_path_to_serve = "index.html".to_string();
        content_type_to_send = "text/html; charset=utf-8"; // Added charset for HTML
    } else {
        // Remove leading '/' to make it a relative path
        let requested_relative_path = path_from_request.trim_start_matches('/').to_string();

        if Path::new(&requested_relative_path).exists() {
            status_line_to_send = "HTTP/1.1 200 OK";
            file_path_to_serve = requested_relative_path.clone(); // Use the relative path

            // Determine content type based on extension
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
            } else if requested_relative_path.ends_with(".heic") {
                // Basic HEIC attempt
                content_type_to_send = "image/heic";
                serve_binary_content = true;
            }
            // Add more image/binary types here as needed (gif, svg, webp, etc.)
            else {
                content_type_to_send = "application/octet-stream"; // Generic binary type
                serve_binary_content = true; // Assume unknown types might be binary
            }
        } else {
            println!(
                "File '{}' not found. Targeting 404.html.",
                requested_relative_path
            );
            status_line_to_send = "HTTP/1.1 404 NOT FOUND";
            file_path_to_serve = "404.html".to_string();
            content_type_to_send = "text/html; charset=utf-8";
            serve_binary_content = false; // 404 page is text
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
                // Simplified 404 for binary read failure
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
        // Serve text-based content (HTML, CSS, JS, or fallback 404.html)
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
                    // If intended file failed, try 404.html
                    status_line_to_send = "HTTP/1.1 404 NOT FOUND"; // Ensure 404 status
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
                    // 404.html itself failed to read
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
