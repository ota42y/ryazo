extern crate diesel;
extern crate ryazo;

use id_store::ryazo::models::Item;
use id_store::ryazo::schema::*;

use std::time::{SystemTime, UNIX_EPOCH};
use self::diesel::prelude::*;
use self::diesel::sqlite::SqliteConnection;
use self::diesel::insert_into;
use self::diesel::*;
use std::env;

pub fn store_hash(item_request_id: &String, item_hash: &String) {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = SqliteConnection::establish(&database_url).unwrap();
    save_hash(&item_request_id, &item_hash, &connection);
}

fn save_hash(item_request_id: &String, item_hash: &String, connection: &SqliteConnection){
    use id_store::ryazo::schema::items::dsl::*;
    use self::diesel::result::Error;

    connection.transaction::<_, Error, _>(|| {
        insert_into(items)
            .values(&vec![
                (hash.eq(item_hash), request_id.eq(item_request_id))
            ])
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

    use id_store::ryazo::schema::items::dsl::*;

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
