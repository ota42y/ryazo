extern crate iron;
extern crate mount;
extern crate multipart;
extern crate ryazo;
extern crate staticfile;

extern crate env_logger;

mod image_store;
mod post_parser;

use image_store::ImageStore;
use iron::prelude::*;
use iron::status;
use mount::Mount;
use multipart::server::{Entries, Multipart, SaveResult};
use staticfile::Static;
use std::env;
use std::error::Error;
use std::io::Write;
use std::net::IpAddr;
use std::path::Path;

fn main() {
    let bind_addr = env::var("BIND_ADDRESS").unwrap().to_string();
    let folder = env::var("SAVE_FOLDER").unwrap().to_string();
    env_logger::init();

    let mut mount = Mount::new();
    mount
        .mount("/", Static::new(Path::new(&folder)))
        .mount("/upload.cgi", process_request);;

    Iron::new(mount).http(bind_addr).expect("Could not bind");
}

fn process_request(request: &mut Request) -> IronResult<Response> {
    let remote_addr = request.remote_addr.ip();

    match Multipart::from_request(request) {
        Ok(mut multipart) => match multipart.save().temp() {
            SaveResult::Full(entries) => process_entries(remote_addr, entries),
            SaveResult::Partial(entries, reason) => {
                process_entries(remote_addr, entries.keep_partial())?;
                Ok(Response::with((
                    status::BadRequest,
                    format!("error reading request: {}", reason.unwrap_err()),
                )))
            }
            SaveResult::Error(error) => Ok(Response::with((
                status::BadRequest,
                format!("error reading request: {}", error),
            ))),
        },
        Err(_) => Ok(Response::with((
            status::BadRequest,
            "The request is not multipart",
        ))),
    }
}

fn process_entries(remote_addr: IpAddr, entries: Entries) -> IronResult<Response> {
    let req = match post_parser::parse_request(remote_addr, entries) {
        Err(e) => return Ok(error_response(e)),
        Ok(d) => d,
    };

    let data = match make_response(&req) {
        Err(e) => return Ok(error_response(e)),
        Ok(d) => d,
    };

    let mut resp = Response::with((status::Ok, data));
    if req.new_id {
        resp.headers
            .append_raw("X-Gyazo-Id", req.id.as_bytes().to_vec());
    }

    Ok(resp)
}

fn error_response(e: Box<Error>) -> Response {
    Response::with((status::BadRequest, format!("error reading request: {}", e)))
}

fn make_response(req: &post_parser::UploadRequest) -> Result<Vec<u8>, Box<Error>> {
    let store = ImageStore::new();
    let filename = store.store_image(&req)?;

    let server_addr = env::var("SERVER_ADDRESS").unwrap().to_string();

    let mut data = Vec::new();
    let _ = writeln!(data, "http://{}/{}", server_addr, filename);

    return Ok(data);
}
