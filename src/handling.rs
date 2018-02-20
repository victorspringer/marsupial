extern crate bodyparser;
extern crate serde;
extern crate persistent;
extern crate bson;
extern crate mongodb;

use iron::status;
use iron::prelude::*;
use router::Router;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use script::Script;
use uploader;

pub fn health(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with(status::Ok))
}

pub fn list_scripts(_req: &mut Request) -> IronResult<Response> {
    let mongo_client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone mongodb client.");

    let sort = mongodb::coll::options::FindOptions {
        sort: Some(doc! {
            "created_at": -1,
            "version": -1,
        }),
        allow_partial_results: false,
        no_cursor_timeout: false,
        oplog_replay: false,
        skip: Some(0),
        limit: None,
        cursor_type: mongodb::coll::options::CursorType::NonTailable,
        batch_size: None,
        comment: None,
        max_time_ms: Some(20000),
        modifiers: None,
        projection: None,
        read_preference: None,
    };

    let coll = mongo_client.db("marsupial").collection("scripts");
    let cursor = coll.find(None, Some(sort.clone())).unwrap();

    let mut scripts = Vec::new();
    let mut last_id = String::from("");
    for result in cursor {
        if let Ok(item) = result {
            let script = bson::from_bson::<Script>(bson::Bson::Document(item)).unwrap();
            if script.id != last_id {
                let script_json = json!({
                    "id": script.id,
                    "version": script.version,
                    "user": script.user,
                    "created_at": script.created_at,
                    "path": script.path,
                    "region": script.region,
                    "aws_key": script.aws_key,
                    "aws_secret": script.aws_secret
                });
                scripts.push(script_json);
            }
            last_id = script.id;
        }
    }

    let res = json!({
        "scripts": scripts
    });
    Ok(Response::with((status::Ok, format!("{}", res))))
}

pub fn get_script_by_id(req: &mut Request) -> IronResult<Response> {
    let ref id = req.extensions.get::<Router>().unwrap().find("id").unwrap_or("/");

    let mongo_client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone mongodb client.");

    let doc = doc! {
        "id": *id
    };

    let sort = mongodb::coll::options::FindOptions {
        sort: Some(doc! {
            "version": -1,
        }),
        allow_partial_results: false,
        no_cursor_timeout: false,
        oplog_replay: false,
        skip: Some(0),
        limit: None,
        cursor_type: mongodb::coll::options::CursorType::NonTailable,
        batch_size: None,
        comment: None,
        max_time_ms: Some(20000),
        modifiers: None,
        projection: None,
        read_preference: None,
    };
    
    let coll = mongo_client.db("marsupial").collection("scripts");
    let cursor = coll.find(Some(doc.clone()), Some(sort.clone())).unwrap();

    let mut scripts = Vec::new();
    for result in cursor {
        if let Ok(item) = result {
            let script = bson::from_bson::<Script>(bson::Bson::Document(item)).unwrap();
            let script_json = json!({
                "id": script.id,
                "version": script.version,
                "user": script.user,
                "created_at": script.created_at,
                "path": script.path,
                "region": script.region,
                "aws_key": script.aws_key,
                "aws_secret": script.aws_secret
            });
            scripts.push(script_json);
        }
    }

    let res = json!({
        "scripts": scripts
    });
    Ok(Response::with((status::Ok, format!("{}", res))))
}

pub fn get_script_by_id_and_version(req: &mut Request) -> IronResult<Response> {
    let ref id = req.extensions.get::<Router>().unwrap().find("id").unwrap_or("/");
    let ref version = req.extensions.get::<Router>().unwrap().find("version").unwrap_or("/");
    let version_int: i64 = version.parse().unwrap();

    let mongo_client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone mongodb client.");

    let doc = doc! {
        "id": *id,
        "version": version_int
    };
    
    let coll = mongo_client.db("marsupial").collection("scripts");
    let mut cursor = coll.find(Some(doc.clone()), None).unwrap();

    let item = cursor.next();
    match item {
        Some(Ok(doc)) => {
            let script = bson::from_bson::<Script>(bson::Bson::Document(doc)).unwrap();
            let res = json!({
                "id": script.id,
                "version": script.version,
                "user": script.user,
                "created_at": script.created_at,
                "code": script.code,
                "language": script.language,
                "path": script.path,
                "region": script.region,
                "aws_key": script.aws_key,
                "aws_secret": script.aws_secret
            });
            Ok(Response::with((status::Ok, format!("{}", res))))
        },
        Some(Err(_)) => Ok(Response::with((status::NotFound, "Script not found."))),
        None => Ok(Response::with((status::NotFound, "Script not found.")))
    }
}

pub fn insert_script(req: &mut Request) -> IronResult<Response> {
    let body = req.get::<bodyparser::Struct<Script>>();

    match body {
        Ok(Some(body)) => {
            let file = uploader::File {
                path: body.path.clone(),
                code: body.code.clone(),
                region: body.region.clone(),
                aws_key: body.aws_key.clone(),
                aws_secret: body.aws_secret.clone()
            };

            let upload = uploader::send_file(file);
            match upload {
                Ok(_) => {},
                Err(err) =>  {
                    let res = json!({
                        "error": format!("{:?}", err)
                    });
                    return Ok(Response::with((status::BadRequest, format!("{}", res))))
                }
            }

            let mongo_client = Client::connect("localhost", 27017)
                .expect("Failed to initialize standalone mongodb client.");

            // get last version
            let sort = mongodb::coll::options::FindOptions {
                sort: Some(doc! {
                    "version": -1,
                }),
                allow_partial_results: false,
                no_cursor_timeout: false,
                oplog_replay: false,
                skip: Some(0),
                limit: Some(1),
                cursor_type: mongodb::coll::options::CursorType::NonTailable,
                batch_size: Some(1),
                comment: None,
                max_time_ms: Some(20000),
                modifiers: None,
                projection: None,
                read_preference: None,
            };

            let find = doc! {
                "path": body.path.clone()
            };

            let coll = mongo_client.db("marsupial").collection("scripts");
            let mut cursor = coll.find(Some(find.clone()), Some(sort.clone())).unwrap();

            let item = cursor.next();
            match item {
                Some(Ok(doc)) => {
                    let script = bson::from_bson::<Script>(bson::Bson::Document(doc)).unwrap();
                    let last_version = script.version;

                    // save new script version
                    let new_doc = doc! {
                        "id": script.id,
                        "version": last_version + 1,
                        "user": body.user,
                        "created_at": body.created_at,
                        "code": body.code,
                        "language": body.language,
                        "path": body.path,
                        "region": body.region,
                        "aws_key": body.aws_key,
                        "aws_secret": body.aws_secret
                    };
                    
                    coll.insert_one(new_doc.clone(), None).ok().expect("Failed to save script.");                   
                    
                    let new_script = bson::from_bson::<Script>(bson::Bson::Document(new_doc)).unwrap();
                    let res = json!({
                        "id": new_script.id,
                        "version": last_version + 1,
                        "user": new_script.user,
                        "created_at": new_script.created_at,
                        "code": new_script.code,
                        "language": new_script.language,
                        "path": new_script.path,
                        "region": new_script.region,
                        "aws_key": new_script.aws_key,
                        "aws_secret": new_script.aws_secret
                    });
                    Ok(Response::with((status::Ok, format!("{}", res))))
                },
                Some(Err(_)) => Ok(Response::with((status::InternalServerError, "Internal server error."))),
                None => {
                    // save new script
                    let new_doc = doc! {
                        "id": body.id,
                        "version": body.version,
                        "user": body.user,
                        "created_at": body.created_at,
                        "code": body.code,
                        "language": body.language,
                        "path": body.path,
                        "region": body.region,
                        "aws_key": body.aws_key,
                        "aws_secret": body.aws_secret
                    };
                    
                    coll.insert_one(new_doc.clone(), None).ok().expect("Failed to save script.");                   
                    
                    let new_script = bson::from_bson::<Script>(bson::Bson::Document(new_doc)).unwrap();
                    let res = json!({
                        "id": new_script.id,
                        "version": body.version,
                        "user": new_script.user,
                        "created_at": new_script.created_at,
                        "code": new_script.code,
                        "language": new_script.language,
                        "path": new_script.path,
                        "region": new_script.region,
                        "aws_key": new_script.aws_key,
                        "aws_secret": new_script.aws_secret
                    });
                    Ok(Response::with((status::Ok, format!("{}", res))))
                }
            }
        },
        Ok(None) => Ok(Response::with((status::BadRequest, "Invalid request body."))),
        Err(err) => Ok(Response::with((status::InternalServerError, format!("Error: {:?}", err))))
    }
}
