use std::{fs, thread};
use env_logger::Env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use clap::{App, Arg};

#[macro_use]
extern crate log;

const PROTOCOL: &str = "HTTP1/1";

struct HttpStatus {
    code: u32,
    msg: &'static str,
}

impl HttpStatus {
    fn full_msg(&self) -> String  {
        format!("{} {}", self.code, self.msg)
    }
}

const HTTP_OK: HttpStatus = HttpStatus{ code: 200, msg: "OK"};
const HTTP_NOT_FOUND: HttpStatus = HttpStatus{ code: 404, msg: "NOT FOUND"};
const HTTP_NOT_ALLOWED: HttpStatus = HttpStatus{ code: 405, msg: "NOT ALLOWED"};

fn main(){
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let matches = App::new("static-server")
        .about("static http server")
        .arg(
            Arg::new("PORT")
                .about("port where the server  will run")
                .default_value("8080")
        )
        .arg(
            Arg::new("FOLDER")
                .about("The path o the folder to be served")
                .default_value(".")
        )
        .get_matches();

    let port = matches.value_of("PORT").unwrap();
    let folder =  String::from(matches.value_of("FOLDER").unwrap());

    info!("Starting server at http://127.0.0.1:{}/", port);

    let listener = TcpListener::bind(format!("localhost:{}", port)).unwrap();
    for stream in listener.incoming(){
        let html_folder = folder.clone();
        let stream = stream.unwrap();
        thread::spawn(move || {
            handle_connection(stream, &html_folder)
        });
    }
}

fn handle_connection(mut stream: TcpStream, html_folder: &str){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request_data = String::from_utf8_lossy(&buffer[..]);
    let request_data_lines: Vec<&str> = request_data.split('\n').collect();
    let first_line = request_data_lines[0];
    let start: Vec<&str> = first_line.split(" ").collect();
    
    let method = start[0];
    let path = start[1];
    
    if method != "GET" {
        let response = format!("{} {}\r\n\r\n", PROTOCOL, HTTP_NOT_ALLOWED.full_msg());
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        info!("{} {} HTTP/1.1 {}",method, path, HTTP_NOT_ALLOWED.code);
        return
    }

    
    let filename = match path {
        "/" => format!("{}index.html", html_folder),
        _ => {
            let filename = path.replacen("/", "", 1);
            format!("{}/{}.html", html_folder, filename)
        }
    };
    

    let contents = match fs::read_to_string(filename) {
        Ok(filename) => filename,
        Err(_e) => {
            let new_path = path.replacen("/", "", 1);
            fs::read_to_string(format!("{}/{}/index.html",html_folder, new_path)).unwrap_or("Not Found".to_string())
        },
    };
    

    let status_line = match contents.as_str() {
        "Not Found" => HTTP_NOT_FOUND.full_msg(),
        _ => HTTP_OK.full_msg(),
    }; 


    let response = format!(
        "{} {}\r\nContent-Length: {}\r\n\r\n{}",
        PROTOCOL,
        status_line,
        contents.len(),
        contents
    );
    
    info!("{} {} {} {}",method, path, PROTOCOL, status_line);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}
