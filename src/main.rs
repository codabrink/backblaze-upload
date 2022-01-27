mod config;

extern crate notify;

use awsregion::Region;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use config::Config;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let config = Config::load().unwrap();
    let _ = std::fs::create_dir_all(&config.upload_dir);

    // Create a channel to receive the events.
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(0)).unwrap();
    watcher
        .watch(&config.upload_dir, RecursiveMode::Recursive)
        .unwrap();

    let creds = Credentials::new(
        Some(&config.access_key),
        Some(&config.secret_key),
        None,
        None,
        None,
    )
    .unwrap();

    let region = Region::Custom {
        region: config.region,
        endpoint: config.endpoint,
    };
    let bucket = Bucket::new(&config.bucket, region, creds).unwrap();
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(p)) => {
                println!("File detected: {:?}", p);
                let bytes = std::fs::read(&p).unwrap();
                let file_name = p.file_name().unwrap().to_string_lossy().to_string();
                let _ = bucket.put_object_blocking(&file_name, &bytes);
                let _ = ctx.set_contents(format!("{}/{}", &config.base_url, &file_name));
                println!("{} uploaded", file_name);
                if config.delete_on_upload {
                    let _ = std::fs::remove_file(p);
                }
            }
            Err(e) => println!("watch error: {:?}", e),
            _ => {}
        }
    }
}
