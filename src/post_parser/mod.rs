extern crate md5;

use std::error::Error;
use std::io::Read;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use multipart::server::Entries;

pub struct UploadRequest {
    pub id: String,
    pub buf: Vec<u8>,
    pub new_id: bool,
}

fn parse_data(entries: Entries) -> Result<UploadRequest, Box<Error>> {
    let r = UploadRequest {
        id: String::new(),
        buf: vec![],
        new_id: false,
    };
    let mut req = r;
    {
        for (name, entries) in &entries.fields {
            let key_name = name.as_str();
            if key_name == "imagedata" {
                for (_idx, field) in entries.iter().enumerate() {
                    let headers = &field.headers;
                    if headers.name.as_str() == "imagedata" {
                        let mut data = field.data.readable().unwrap();
                        data.read_to_end(&mut req.buf)?;
                    }
                }
            }
            if key_name == "id" {
                for (_idx, field) in entries.iter().enumerate() {
                    let headers = &field.headers;
                    if headers.name.as_str() == "id" {
                        let mut data = field.data.readable().unwrap();
                        data.read_to_string(&mut req.id)?;
                    }
                }
            }
        }
    }
    return Ok(req);
}

fn create_new_id(remote_addr: IpAddr, r: &mut UploadRequest) {
    let sec = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let sec_str = sec.subsec_nanos().to_string();
    let digest = md5::compute(format!("{}{}", remote_addr.to_string(), sec_str));
    r.id = format!("{:x}", digest);
    r.new_id = true;
}

pub fn parse_request(remote_addr: IpAddr, entries: Entries) -> Result<UploadRequest, Box<Error>> {
    let mut req = parse_data(entries)?;

    if req.id == "" {
        create_new_id(remote_addr, &mut req);
    }

    return Ok(req);
}

#[test]
fn test_new_id() {
    use std::net::Ipv4Addr;

    let localhost_v4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let mut r = UploadRequest {
        id: String::new(),
        buf: vec![],
        new_id: false,
    };

    assert_eq!(r.id, String::new());

    create_new_id(localhost_v4, &mut r);

    assert_eq!(r.new_id, true);
    assert_ne!(r.id, String::new());
}
