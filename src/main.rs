extern crate iron;
extern crate mount;
extern crate multipart;
extern crate ryazo;
extern crate staticfile;

extern crate env_logger;

mod image_store;
mod post_parser;

use iron::prelude::*;
use iron::status;
use mount::Mount;
use multipart::mock::StdoutTee;
use multipart::server::{Entries, Multipart, SaveResult};
use self::post_parser::UploadRequest;
use staticfile::Static;
use std::env;
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
 qq   }
}

fn process_entries(remote_addr: IpAddr, entries: Entries) -> IronResult<Response> {
    let mut req = post_parser::parse_request(remote_addr, entries);

    let data = make_response(&req);
    let mut resp = Response::with((status::Ok, data));
    resp.headers
        .append_raw("X-Gyazo-Id", req.id.as_bytes().to_vec());

    Ok(resp)
}

fn make_response(req: &post_parser::UploadRequest) -> Vec<u8> {
    let folder = env::var("SAVE_FOLDER").unwrap().to_string();
    let filename = image_store::store_image(&req, &folder);

    let server_addr = env::var("SERVER_ADDRESS").unwrap().to_string();

    let mut data = Vec::new();
    let _ = writeln!(data, "http://{}/{}", server_addr, filename);

    return data;
}
