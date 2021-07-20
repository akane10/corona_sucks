#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use rocket::request::Request;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket_cors;
use std::error::Error;
// use std::fs;
// use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[get("/")]
fn list() -> Result<Json<Vec<Value>>, Box<dyn Error>> {
    let mut list = Vec::new();
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("public/data/data.json");
    let file = File::open(&p)?;
    let reader = BufReader::new(file);
    let data: HashMap<u64, Value> = serde_json::from_reader(reader)?;
    for (key, val) in data {
        list.push(json!({ "sheet_id": key, "title": val["title"], "total_row": val["total_row"] }));
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
