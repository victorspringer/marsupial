extern crate iron;
extern crate router;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate s3;

use iron::prelude::*;
use router::Router;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::IndexOptions;

mod handling;
mod uploader;
mod script;

fn main() {
    let mongo_client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone mongodb client.");
    let coll = mongo_client.db("marsupial").collection("scripts");
    coll.create_index(
        doc! {"version": -1},
        Some(<IndexOptions>::new())
    ).unwrap();
    coll.create_index(
        doc! {"created_at": -1, "version": -1},
        Some(<IndexOptions>::new())
    ).unwrap();
    coll.create_index(
        doc! {"path": 1},
        Some(<IndexOptions>::new())
    ).unwrap();

    let mut router = Router::new();
    router.get("/health", handling::health, "health");
    router.get("/resource-status", handling::health, "resource-status");
    router.get("/list-scripts", handling::list_scripts, "list-scripts");
    router.get("/get-script/:id", handling::get_script_by_id, "get-script");
    router.get(
        "/get-script-version/:id/:version",
        handling::get_script_by_id_and_version,
        "get-script-version"
    );
    router.post("/insert-script", handling::insert_script, "insert-script");

    Iron::new(router).http("localhost:8080").unwrap();
}
