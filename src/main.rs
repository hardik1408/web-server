use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

struct Response {
    status_line: String,
    contents: String,
}

impl Response {
    fn length(&self) -> usize {
        self.contents.len()
    }

    fn string(&self) -> String {
        format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_line,
            self.length(),
            self.contents
        )
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7567").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream)
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next().unwrap() {
        Ok(line) => line, // Req line of the form: "GET / HTTP/1.1"
        Err(_) => {
            println!("There was an error reading the request");
            String::from("bruh")
        }
    };
    let requested_path = request_line.trim().split_whitespace().nth(1);

    let response = match requested_path {
        Some(req_path) => {
            let mut req_path = "root".to_string() + req_path;

            if req_path.chars().last().unwrap() == '/' {
                let test_path = req_path.clone() + "index.html";
                match Path::new(&test_path).try_exists() {
                    Ok(true) => {
                        req_path = test_path;
                    }
                    _ => {}
                }
            }
            match fs::read_to_string(&req_path) {
                Ok(file_contents) => Response {
                    status_line: "HTTP/1.1 200 OK".to_string(),
                    contents: file_contents,
                },
                Err(_) => match Path::new(&req_path).try_exists() {
                    Ok(true) => Response {
                        status_line: "HTTP/1.1 403 Forbidden".to_string(),
                        contents: fs::read_to_string("root/403.html").unwrap(),
                    },
                    Ok(false) => Response {
                        status_line: "HTTP/1.1 404 Not Found".to_string(),
                        contents: fs::read_to_string("root/404.html").unwrap(),
                    },
                    Err(_) => Response {
                        status_line: "HTTP/1.1 500 Internal Server Error".to_string(),
                        contents: fs::read_to_string("root/500.html").unwrap(),
                    },
                },
            }
        }
        None => Response {
            status_line: "HTTP/1.1 400 Bad Request".to_string(),
            contents: String::new(),
        },
    };

    match stream.write_all(response.string().as_bytes()) {
        Ok(_) => match requested_path {
            Some(req_path) => {
                println!("Responded to request to {}", req_path);
            }
            None => {
                println!("Request was malformed, responded with a 400: Bad Request");
            }
        },
        Err(_) => {
            println!(
                "Could not respond to request to {}",
                requested_path.unwrap()
            );
        }
    };
}
