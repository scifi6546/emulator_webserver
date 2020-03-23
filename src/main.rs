use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryStreamExt};
#[macro_use]
extern crate diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::io::Write;
pub mod models;
pub mod schema;
use self::models::*;
use schema::files::dsl::*;
use std::sync::Mutex;
use std::sync::*;
struct State {
    db_connection: SqliteConnection,
}
impl State {
    pub fn new(db_path: &str) -> State {
        State {
            db_connection: SqliteConnection::establish(db_path).expect("bad"),
        }
    }
}
async fn save_file(
    mut payload: Multipart,
    data: web::Data<std::sync::Mutex<State>>,
) -> Result<HttpResponse, Error> {
    println!("saving file");
    let db_conn = &data.lock().unwrap().db_connection;
    // iterate over multipart stream
    let mut filename = String::new();
    let mut upload = false;
    let mut file_id = 0;
    while let Ok(Some(mut field)) = payload.try_next().await {
        println!("file?");
        let content_type = field.content_disposition().unwrap();
        filename = content_type.get_filename().unwrap().to_string();
        let filepath = format!("./tmp/{}", filename);

        let res = files
            .filter(path.eq(filename))
            .limit(1)
            .load::<File>(db_conn).expect("failed");
            
        
        if res.len() == 1 {
            if res[0].in_filesystem == false {
                upload = true;
                file_id = res[0].id;
            }
        }
        // File::create is blocking operation, use threadpool
        if upload {
            let mut f = web::block(|| std::fs::File::create(filepath))
                .await
                .unwrap();
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                // filesystem operations are blocking, we have to use threadpool
                f = web::block(move || f.write_all(&data).map(|_| f)).await?;
            }
        }
    }
    if upload==true{
        diesel::update(files.find(file_id)).set(in_filesystem.eq(true)).execute(db_conn);
    }

    Ok(HttpResponse::Ok().into())
}
fn file_index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/upload/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <input type="submit" value="Submit"></button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}
async fn get_files(data: web::Data<std::sync::Mutex<State>>) -> impl Responder {
    let db_con = &data.lock().unwrap().db_connection;
    let res = files
        .filter(in_filesystem.eq(true))
        .limit(10)
        .load::<File>(db_con)
        .expect("crap");
    let mut out_str = "In Filesystem:\n".to_string();
    for file in res {
        out_str += &format!(
            "id: {}\n\tfile name: {}\n\tin_filesystem: {}\n",
            file.id, file.path, file.in_filesystem
        );
    }
    let res2 = files
        .filter(in_filesystem.eq(false))
        .limit(10)
        .load::<File>(db_con)
        .expect("crap");
    out_str += "Not In Filesystem:\n";
    for file in res2 {
        out_str += &format!(
            "id: {}\n\tfile name: {}\n\tin_filesystem: {}\n",
            file.id, file.path, file.in_filesystem
        );
    }
    //let name = req.match_info().get("name").unwrap_or("World");
    format!("files \n{}", &out_str)
}
async fn add_file(data: web::Data<std::sync::Mutex<State>>, req: HttpRequest) -> impl Responder {
    use schema::files;
    let t_path = req.match_info().get("path").unwrap_or("World").to_string();
    let new_file = NewFile {
        path: t_path.clone(),
        in_filesystem: false,
    };
    let mut db_con = &data.lock().unwrap().db_connection;
    diesel::insert_into(files::table)
        .values(&new_file)
        .execute(db_con)
        .expect("error saving");
    format!("Hello {}!", &t_path)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //d.bar();
    HttpServer::new(move || {
        App::new()
            .data(Mutex::new(State::new("test.sqlite3")))
            .route("/", web::get().to(get_files))
            .route("/add/{path}", web::get().to(add_file))
            .route("/upload_html.html", web::get().to(file_index))
            .service(
                web::resource("/upload")
                    .route(web::post().to(save_file))
                    .route(web::get().to(file_index)),
            )
            .route("/", web::post().to(save_file))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
