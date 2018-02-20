extern crate s3;

use std::str;
use s3::bucket::Bucket;
use s3::credentials::Credentials;
use s3::error::S3Error;

pub struct File {
    pub path: String,
    pub code: String,
    pub region: String,
    pub aws_key: String,
    pub aws_secret: String
}

pub fn send_file(file: File) -> Result<(), S3Error> {
    let credentials = Credentials::new(
        Some(file.aws_key),
        Some(file.aws_secret),
        None,
        None
    );

    let filepath = file.path.split("/");
    let mut path = filepath.collect::<Vec<&str>>();

    let region = file.region.parse()?;

    let bucket = Bucket::new(path[0], region, credentials);

    path.remove(0);
    let ref script = path.join("/");

    let (_, _) = bucket.delete(script)?;

    let (_, _) = bucket.put(script, file.code.as_bytes(), "application/octet-stream")?;

    Ok(())
}
