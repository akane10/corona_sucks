#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use rocket::request::Request;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket_cors;
use std::error::Error;
use std::fs;
// use serde::{Deserialize, Serialize};
// use serde_json::json;
// use serde_json::Value;

#[get("/")]
fn list() -> Result<Json<Vec<String>>, Box<dyn Error>> {
    let mut list = Vec::new();
    let paths = fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/public/data"))?;
    for path in paths {
        let p = path?.file_name().to_str().unwrap().to_string();
        let split = p.split("/");
        let vec: Vec<&str> = split.collect();
        let file = vec[vec.len() - 1].to_string();
        if file.ne("lastest_updated.json") {
            list.push(file);
        }
    }

    Ok(Json(list))
}

#[catch(500)]
fn internal_error() -> &'static str {
    "Whoops! Looks like we messed up."
}
#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Couldn't find '{}'. Try something else?", req.uri())
}
pub fn rocket_app() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/public")),
        )
        .mount("/list", routes![list])
        .attach(rocket_cors::CorsOptions::default().to_cors().unwrap())
        .register(catchers![not_found, internal_error])
}
