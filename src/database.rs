extern crate rand;
extern crate crypto;

use rand::random;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use crate::models::{UserAdd, User};
use crate::schema::*;
use crate::crypto_functional::to_base64;

use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;


/// Generates hash for the string. All password must go through this function
fn get_hash(text: &str) -> String {
    let mut buff_str = text.to_string();
    for _x in 0..512 {
        let mut hasher = Sha256::new();
        hasher.input_str(&buff_str);
        buff_str = hasher.result_str()
    }

    return buff_str;
}

/// Generates random token for the user
pub fn get_random_token() -> String {
    get_hash(&(0..32).map(|_| random::<char>()).collect::<String>())
}

#[derive(Clone)]
pub struct DBManager {
    pool: Pool,
}

impl DBManager {
    pub fn open(db_path: &str) -> Result<Self, String> {
        let manager = ConnectionManager::<SqliteConnection>::new(db_path);
        match Pool::builder().build(manager) {
            Ok(pool) => Ok(DBManager { pool }),
            Err(err) => Err(format!("Error on getting connection to DB: {:?}", err))
        }
    }

    pub fn init(db_path: &str) -> Result<(), String> {
        let buf = DBManager::open(db_path)?;
        let connection = match buf.pool.get() {
            Ok(t) => t,
            Err(err) => return Err(format!("Error on buf.pool.get() to db in DBManager::init: {:?}", err)),
        };

        match connection.batch_execute(r#"
        CREATE TABLE "users" (
            "id"	INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
            "name"	TEXT NOT NULL UNIQUE,
            "key"	TEXT NOT NULL
        )
        "#) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Error on connection.batch_execute to db in DBManager::init: {:?}", err)),
        }
    }

    pub fn update_key(&self, username: &str, key: &Vec<u8>) -> Result<(), String> {
        let connection = match self.pool.get() {
            Ok(conn) => conn,
            Err(err) => return Err(format!("Error on update_user_pass (connection): {:?}", err)),
        };

        match diesel::update(users::table.filter(users::columns::name.eq(username)))
            .set(users::columns::key.eq(to_base64(key)))
            .execute(&connection) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Error on update_user_pass (update): {:?}", err))
        }
    }

    pub fn insert_user(&self, username: &str, key: &Vec<u8>) -> Result<(), String> {
        let connection = match self.pool.get() {
            Ok(conn) => conn,
            Err(err) => return Err(format!("Error on insert_user (connection): {:?}", err)),
        };

        let key_based = to_base64(key);
        let buffer = UserAdd { name: username, key: &key_based };

        match diesel::insert_into(users::table)
            .values(&buffer)
            .execute(&connection) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Error on inserting users (insert): {:?}", err))
        }
    }

    pub fn get_user(&self, username: &str) -> Result<User, String> {
        let connection = match self.pool.get() {
            Ok(conn) => conn,
            Err(err) => return Err(format!("Error on get_user (connection): {:?}", err)),
        };

        match users::table.filter(users::columns::name.eq(username)).first::<User>(&connection) {
            Ok(r) => Ok(r),
            Err(e) => {
                eprintln!("Error on getting user from cookie: {:?}", e);
                return Err(format!("Error on getting user from cookie: {:?}", e));
            }
        }
    }
}