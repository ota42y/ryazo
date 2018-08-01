extern crate diesel;
extern crate md5;
extern crate ryazo;

use std::fs::File;

use image_store::ryazo::models::Item;
use image_store::ryazo::schema::*;
use post_parser::UploadRequest;

use self::diesel::insert_into;
use self::diesel::prelude::*;
use self::diesel::sqlite::SqliteConnection;
use self::diesel::*;
use std::env;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn store_image(req: &UploadRequest, folder: &String) -> String {
    let digest = format!("{:x}", md5::compute(&req.buf));
    let filename = format!("{}.png", digest);

    save_file(&req.buf, &filename, &folder);
    store_hash(&req.id, &digest);

    return filename;
}

// return filename, digest
fn save_file(image_buf: &Vec<u8>, filename: &String, folder: &String) {
    let path = format!("{}/{}", folder, filename);

    let mut buffer = File::create(path).unwrap();
    buffer.write(image_buf);
}

fn store_hash(item_request_id: &String, item_hash: &String) {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = SqliteConnection::establish(&database_url).unwrap();
    save_hash(&item_request_id, &item_hash, &connection);
}

fn save_hash(item_request_id: &String, item_hash: &String, connection: &SqliteConnection) {
    use self::diesel::result::Error;
    use image_store::ryazo::schema::items::dsl::*;

    connection.transaction::<_, Error, _>(|| {
        insert_into(items)
            .values(&vec![(hash.eq(item_hash), request_id.eq(item_request_id))])
            .execute(connection)?;

        Ok(true)
    });
}

#[test]
fn test_save_hash() {
    let connection = SqliteConnection::establish("./data/ryazo_test.db").unwrap();
    let item_request_id = "xyz".to_string();
    let item_hash = "abc".to_string();
    save_hash(&item_request_id, &item_hash, &connection);

    use image_store::ryazo::schema::items::dsl::*;

    let results = items
        .filter(hash.eq(&item_hash))
        .limit(1)
        .load::<Item>(&connection)
        .expect("Error loading posts");

    assert_eq!(results.len(), 1);

    let first_item = &results[0];
    assert_eq!(first_item.hash, item_hash);
    assert_eq!(first_item.request_id, item_request_id);
}
