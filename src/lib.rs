#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use rocket::request::Request;
// use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
// use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};
// use std::error::Error;

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
        .register(catchers![not_found, internal_error])
}
