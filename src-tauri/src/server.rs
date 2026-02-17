use crate::config::{get, set};
use crate::window::*;
use log::{info, warn};
use std::panic;
use std::thread;
use tiny_http::{Request, Response, Server};

pub fn start_server() {
    let port = match get("server_port") {
        Some(v) => v.as_i64().unwrap(),
        None => {
            set("server_port", 60828);
            60828
        }
    };
    info!("Starting HTTP server on 127.0.0.1:{}", port);
    thread::spawn(move || {
        let server = match Server::http(format!("127.0.0.1:{port}")) {
            Ok(v) => {
                info!("HTTP server started successfully on port {}", port);
                v
            }
            Err(e) => {
                warn!("Server start failed on port {}: {}", port, e);
                return;
            }
        };
        for request in server.incoming_requests() {
            let url = request.url().to_string();
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                http_handle(request);
            }));
            if let Err(e) = result {
                warn!("Handler panicked for {}: {:?}", url, e);
            }
        }
    });
}

fn http_handle(request: Request) {
    info!("Handle {} request", request.url());
    match request.url() {
        "/" => handle_translate(request),
        "/config" => handle_config(request),
        "/translate" => handle_translate(request),
        "/selection_translate" => handle_selection_translate(request),
        "/input_translate" => handle_input_translate(request),
        "/ocr_recognize" => handle_ocr_recognize(request),
        "/ocr_translate" => handle_ocr_translate(request),
        "/ocr_recognize?screenshot=false" => handle_ocr_recognize(request),
        "/ocr_translate?screenshot=false" => handle_ocr_translate(request),
        "/ocr_recognize?screenshot=true" => handle_ocr_recognize(request),
        "/ocr_translate?screenshot=true" => handle_ocr_translate(request),
        _ => warn!("Unknown request url: {}", request.url()),
    }
}

fn handle_config(request: Request) {
    config_window();
    response_ok(request);
}

fn handle_translate(mut request: Request) {
    let mut content = String::new();
    if let Err(e) = request.as_reader().read_to_string(&mut content) {
        warn!("Failed to read request body: {}", e);
    }
    text_translate(content);
    response_ok(request);
}

fn handle_selection_translate(request: Request) {
    selection_translate();
    response_ok(request);
}

fn handle_input_translate(request: Request) {
    input_translate();
    response_ok(request);
}

fn handle_ocr_recognize(request: Request) {
    if request.url().ends_with("false") {
        recognize_window();
    } else {
        ocr_recognize();
    }
    response_ok(request);
}

fn handle_ocr_translate(request: Request) {
    if request.url().ends_with("false") {
        image_translate();
    } else {
        ocr_translate();
    }
    response_ok(request);
}

fn response_ok(request: Request) {
    let response = Response::from_string("ok");
    if let Err(e) = request.respond(response) {
        warn!("Failed to send response: {}", e);
    }
}
