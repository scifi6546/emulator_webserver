use super::schema::files;
#[derive(Queryable)]
pub struct File {
    pub id: i32,
    pub path: String,
    pub in_filesystem: bool,
}
#[derive(Insertable)]
#[table_name = "files"]
pub struct NewFile {
    pub path: String,
    pub in_filesystem: bool,
}
