extern crate chrono;
extern crate diesel;

use self::chrono::NaiveDateTime;

#[derive(Queryable)]
pub struct Item {
    pub id: i32,
    pub hash: String,
    pub request_id: String,
    pub created_at: NaiveDateTime,
}
