extern crate multipart;
extern crate iron;
extern crate md5;
extern crate staticfile;
extern crate mount;

extern crate env_logger;

mod post_parser;
mod id_store;

use std::path::Path;
use mount::Mount;
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::fs::File;
use std::io::{self, Write};
use multipart::mock::StdoutTee;
use multipart::server::{Multipart, Entries, SaveResult};
use iron::prelude::*;
use iron::status;
use std::env;
use staticfile::Static;


fn main() {
    let bind_addr = env::var("BIND_ADDRESS").unwrap().to_string();
    let folder = env::var("SAVE_FOLDER").unwrap().to_string();
    env_logger::init();

    let mut mount = Mount::new();
    mount.mount("/", Static::new(Path::new(&folder))).mount("/upload.cgi", process_request);;

    Iron::new(mount).http(bind_addr).expect("Could not bind");
}

fn process_request(request: &mut Request) -> IronResult<Response> {
    let remote_addr = request.remote_addr.ip();

    match Multipart::from_request(request) {
        Ok(mut multipart) => {
            match multipart.save().temp() {
                SaveResult::Full(entries) => process_entries(remote_addr, entries),
                SaveResult::Partial(entries, reason) => {
                    process_entries(remote_addr,entries.keep_partial())?;
                    Ok(Response::with((
                        status::BadRequest,
                        format!("error reading request: {}", reason.unwrap_err())
                    )))
                }
                SaveResult::Error(error) => Ok(Response::with((
                    status::BadRequest,
                    format!("error reading request: {}", error)
                ))),
            }
        }
        Err(_) => {
            Ok(Response::with((status::BadRequest, "The request is not multipart")))
        }
    }
}


fn save_file(req: post_parser::UploadRequest) -> String{
    let folder = env::var("SAVE_FOLDER").unwrap().to_string();

    let digest = format!("{:x}", md5::compute(&req.buf));
    let filename = format!("{}.png", digest);
    let path = format!("{}/{}", folder, filename);

    let mut buffer = File::create(path).unwrap();
    buffer.write(&req.buf);

    id_store::store_hash(&req.id,&digest);
    println!("{{\"id\":\"{}\", \"hash\":\"{}\"}}", req.id, digest);

    return filename;
}

fn process_entries(remote_addr: IpAddr, entries: Entries) -> IronResult<Response> {
    let server_addr = env::var("SERVER_ADDRESS").unwrap().to_string();

    let mut data = Vec::new();
    let mut req = post_parser::parse_request(remote_addr, entries);

    let id = req.id.clone();

    let filename = save_file(req);

    let _ = writeln!(data, "http://{}/{}", server_addr, filename);

    let mut resp = Response::with((status::Ok, data));
    resp.headers.append_raw("X-Gyazo-Id", id.as_bytes().to_vec());

    Ok(resp)
}
