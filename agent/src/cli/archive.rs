use flate2::{write::GzEncoder, Compression};
use rocket::tokio::task;
use std::{fs, fs::File, future::Future, io::Error, path::Path};
use tar::Builder;
use urlencoding::decode;

pub struct ArchiveItem {
    pub source: String,
    pub destination: String,
}

pub struct ArchiveError {
    pub code: i32,
    pub message: String,
}

pub struct ArchiveWriter {
    compress: bool,
    gzip_writer: Option<Builder<GzEncoder<File>>>,
    tar_writer: Option<Builder<File>>,
}

impl ArchiveWriter {
    pub fn new(archive_path: &str, compress: bool) -> Result<Self, ArchiveError> {
        let file = match File::create(archive_path) {
            Ok(f) => f,
            Err(_) => {
                return Err(ArchiveError {
                    code: 300,
                    message: format!("Cannot open {archive_path} for writing"),
                });
            }
        };

        return if compress {
            let enc = GzEncoder::new(file, Compression::default());
            let writer = Builder::new(enc);
            Ok(Self {
                compress,
                tar_writer: None,
                gzip_writer: Some(writer),
            })
        } else {
            let writer = Builder::new(file);
            Ok(Self {
                compress,
                tar_writer: Some(writer),
                gzip_writer: None,
            })
        };
    }

    pub fn crate_archive(
        &mut self,
        items: Vec<ArchiveItem>,
    ) -> impl Future<Output = Result<(), ArchiveError>> + '_ {
        async move {
            for item in items.iter() {
                let src_ = decode(&item.source).unwrap().into_owned();
                let dst_ = decode(&item.destination).unwrap().into_owned();
                let src = String::from(src_.replacen("/files", "/srv", 1));
                let dst = String::from(dst_.trim_start_matches("/"));

                let res = match self.add_file_to_archive(src.clone(), dst) {
                    Ok(_) => Ok::<(), Error>(()),
                    Err(e) => {
                        return Err(ArchiveError {
                            code: 301,
                            message: format!("Couldn't add {src} to archive: {}", e.to_string()),
                        });
                    }
                };

                if res.is_err() {
                    return Err(ArchiveError {
                        code: 302,
                        message: res.unwrap_err().to_string(),
                    });
                }

                task::yield_now().await;
            }

            Ok(())
        }
    }

    fn add_file_to_archive(&mut self, src: String, path: String) -> Result<(), Error> {
        let src_path = Path::new(src.as_str());
        let src_meta = match src_path.metadata() {
            Ok(m) => m,
            Err(e) => return Err(e)
        };

        if src_meta.is_dir() {
            // walk path and recurse on items
            for item in fs::read_dir(src_path)? {
                let item = item?;
                let item_fn = item
                    .path()
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap();
                let item_path = item.path().into_os_string().into_string().unwrap();
                let item_dst = format!("{}/{}", path, item_fn);

                self.add_file_to_archive(item_path, item_dst)?;
            }
        }

        if !src_meta.file_type().is_file() {
            return Ok(());
        }

        let res = match self.compress {
            true => self
                .gzip_writer
                .as_mut()
                .unwrap()
                .append_path_with_name(src, path),

            false => self
                .tar_writer
                .as_mut()
                .unwrap()
                .append_path_with_name(src, path),
        };

        if res.is_err() {
            Err(res.unwrap_err())
        } else {
            Ok(())
        }
    }
}
