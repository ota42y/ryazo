extern crate diesel;
extern crate md5;
extern crate ryazo;

use std::error::Error;
use std::fs::File;

use post_parser::UploadRequest;

use self::diesel::insert_into;
use self::diesel::prelude::*;
use self::diesel::sqlite::SqliteConnection;
use std::env;
use std::io::Write;

pub struct ImageStore {
    folder: String,
    database_url: String,
}

impl ImageStore {
    pub fn new() -> ImageStore {
        let folder = env::var("SAVE_FOLDER").unwrap().to_string();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        return ImageStore {
            folder: folder,
            database_url: database_url,
        };
    }

    pub fn store_image(&self, req: &UploadRequest) -> Result<String, Box<Error>> {
        let digest = format!("{:x}", md5::compute(&req.buf));
        let filename = format!("{}.png", digest);

        self.save_file(&req.buf, &filename)?;
        self.store_hash(&req.id, &digest)?;

        return Ok(filename);
    }

    fn save_file(&self, image_buf: &Vec<u8>, filename: &String) -> Result<bool, Box<Error>> {
        let path = format!("{}/{}", self.folder, filename);

        let mut buffer = File::create(path)?;
        buffer.write(image_buf)?;

        Ok(true)
    }

    fn store_hash(&self, item_request_id: &String, item_hash: &String) -> Result<bool, Box<Error>> {
        let connection = SqliteConnection::establish(&self.database_url).unwrap();
        return self.save_hash(&item_request_id, &item_hash, &connection);
    }

    fn save_hash(
        &self,
        item_request_id: &String,
        item_hash: &String,
        connection: &SqliteConnection,
    ) -> Result<bool, Box<Error>> {
        use self::diesel::result::Error;
        use image_store::ryazo::schema::items::dsl::*;

        connection.transaction::<_, Error, _>(|| {
            insert_into(items)
                .values(&vec![(hash.eq(item_hash), request_id.eq(item_request_id))])
                .execute(connection)?;
            Ok(true)
        })?;

        Ok(true)
    }
}

#[test]
fn test_save_hash() {
    use image_store::ryazo::models::Item;

    let store = ImageStore {
        folder: "folder".to_string(),
        database_url: "./data/ryazo_test.db".to_string(),
    };

    let item_request_id = "xyz".to_string();
    let item_hash = "abc".to_string();
    let _ = store.store_hash(&item_request_id, &item_hash);

    use image_store::ryazo::schema::items::dsl::*;

    let connection = SqliteConnection::establish(&store.database_url).unwrap();
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
