use crate::schema::*;

#[derive(Queryable, PartialEq, Debug, Identifiable, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub key: String,
}

#[derive(Deserialize, Insertable)]
#[table_name = "users"]
pub struct UserAdd<'a> {
    pub name: &'a str,
    pub key: &'a str
}