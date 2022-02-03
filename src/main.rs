mod config;

extern crate notify;

use awsregion::Region;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use config::Config;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{ffi::OsStr, io::Cursor};

use rodio::{source::Source, Decoder, OutputStream};

fn main() {
  let config = Config::load().unwrap();
  let _ = std::fs::create_dir_all(&config.upload_dir);

  // Create a channel to receive the events.
  let (tx, rx) = channel();
  let mut watcher = watcher(tx, Duration::from_millis(200)).unwrap();
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

  let bell = include_bytes!("bird.ogg");
  let (_stream, stream_handle) = OutputStream::try_default().unwrap();

  loop {
    match rx.recv() {
      Ok(DebouncedEvent::Create(p)) if p.is_file() => {
        println!("File detected: {:?}", p);
        let bytes = std::fs::read(&p).unwrap();

        let file_name = p.file_name().unwrap();
        let file_extension = p.extension().unwrap_or(OsStr::new("")).to_str().unwrap();
        let obfuscated_file = format!("{}.{}", rand_string(5), file_extension);

        let _ = match mime_guess::from_ext(file_extension).first() {
          Some(guess) => bucket.put_object_with_content_type_blocking(
            &obfuscated_file,
            &bytes,
            &guess.to_string(),
          ),
          _ => bucket.put_object_blocking(&file_name.to_str().unwrap(), &bytes),
        };

        let _ = ctx.set_contents(format!("{}/{}", &config.base_url, &obfuscated_file));

        println!("{:?} uploaded", file_name);
        if config.delete_on_upload {
          let _ = std::fs::remove_file(p);
        }

        if config.upload_sound {
          let reader = Cursor::new(bell);
          let source = Decoder::new(reader).unwrap();
          let _ = &stream_handle.play_raw(source.convert_samples());
        }
      }
      Err(e) => println!("watch error: {:?}", e),
      _ => {}
    }
  }
}

fn rand_string(len: usize) -> String {
  thread_rng()
    .sample_iter(&Alphanumeric)
    .take(len)
    .map(char::from)
    .collect()
}
