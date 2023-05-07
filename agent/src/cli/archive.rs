use flate2::{write::GzEncoder, Compression};
use rocket::tokio::task;
use std::{
    fs,
    fs::File,
    io::Error,
    path::Path,
    time::{Duration, Instant},
};
use tar::Builder;
use urlencoding::decode;

use crate::files_api::{FilesApi, Transfer};

pub struct ArchiveItem {
    pub source: String,
    pub destination: String,
}

#[derive(Clone)]
pub struct ArchiveError {
    pub code: i32,
    pub message: String,
}

pub struct ArchiveWriter {
    compress: bool,
    file_path: String,
    gzip_writer: Option<Builder<GzEncoder<File>>>,
    tar_writer: Option<Builder<File>>,
    source_base_path: String,
    progress: ProgressCounter,
    last_update_sent_at: Instant,
}

pub struct ProgressCounter {
    items_total: usize,
    items_added: usize,
    files_added: usize,
}

impl Clone for ProgressCounter {
    fn clone(&self) -> Self {
        Self {
            items_total: self.items_total,
            items_added: self.items_added,
            files_added: self.files_added,
        }
    }
}

impl ArchiveWriter {
    pub fn new(
        archive_path: &str,
        compress: bool,
        source_base_path: &str,
    ) -> Result<Self, ArchiveError> {
        let file = match File::create(archive_path) {
            Ok(f) => f,
            Err(_) => {
                return Err(ArchiveError {
                    code: 300,
                    message: format!("Cannot open {archive_path} for writing"),
                });
            }
        };

        if compress {
            let enc = GzEncoder::new(file, Compression::fast());
            let writer = Builder::new(enc);
            Ok(Self {
                compress,
                file_path: archive_path.to_string(),
                tar_writer: None,
                gzip_writer: Some(writer),
                source_base_path: String::from(source_base_path),
                progress: ProgressCounter {
                    items_total: 0,
                    items_added: 0,
                    files_added: 0,
                },
                last_update_sent_at: Instant::now(),
            })
        } else {
            let writer = Builder::new(file);
            Ok(Self {
                compress,
                file_path: archive_path.to_string(),
                tar_writer: Some(writer),
                gzip_writer: None,
                source_base_path: String::from(source_base_path),
                progress: ProgressCounter {
                    items_total: 0,
                    items_added: 0,
                    files_added: 0,
                },
                last_update_sent_at: Instant::now(),
            })
        }
    }

    pub async fn crate_archive<'a>(
        &'a mut self,
        items: Vec<ArchiveItem>,
        transfer: &'a Transfer,
    ) -> Result<(), ArchiveError> {
        self.progress.items_total = items.len();

        // loop through submitted items adding each to the archive
        for item in items.iter() {
            let src_ = decode(&item.source).unwrap().into_owned();
            let dst_ = decode(&item.destination).unwrap().into_owned();
            let src = src_.replacen("/files", &self.source_base_path, 1);
            let dst = String::from(dst_.trim_start_matches('/'));
            self.progress.items_added += 1;

            let res = match self.add_file_to_archive(src.clone(), dst, transfer) {
                Ok(_) => Ok::<(), Error>(()),
                Err(e) => {
                    return Err(ArchiveError {
                        code: 301,
                        message: e.to_string(),
                    });
                }
            };

            if let Err(err) = res {
                return Err(ArchiveError {
                    code: 302,
                    message: err.to_string(),
                });
            }

            task::yield_now().await;

            // send progress update
            let msg = &format!(
                "progress::{}::{}/{}/{}",
                self.get_job_type(),
                self.progress.items_added,
                self.progress.items_total,
                self.progress.files_added,
            );
            FilesApi::new()
                .send_upload_status_update_async(transfer, msg)
                .await;
        }

        Ok(())
    }

    fn remove_archive(&self) {
        let _ = fs::remove_file(&self.file_path);
    }

    fn get_job_type(&self) -> &'static str {
        match self.compress {
            true => "compressed",
            false => "archived",
        }
    }

    fn add_file_to_archive(
        &mut self,
        src: String,
        path: String,
        transfer: &Transfer,
    ) -> Result<(), Error> {
        let src_path = Path::new(src.as_str());
        let src_meta = match src_path.metadata() {
            Ok(m) => m,
            Err(e) => return Err(e),
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

                self.add_file_to_archive(item_path, item_dst, transfer)?;
            }
        }

        // skip non regular files
        if !src_meta.file_type().is_file() {
            return Ok(());
        }

        // try adding the file
        let src_copy = src.clone();
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

        if let Err(err) = res {
            self.remove_archive();
            // include problem file path in error message
            let err_msg = format!("{} {}", err, src_copy);
            Err(Error::new(err.kind(), err_msg))
        } else {
            self.progress.files_added += 1;

            // send progress updates not more frequently than one in every 3 seconds
            if self.last_update_sent_at.elapsed() > Duration::from_secs(3) {
                self.last_update_sent_at = Instant::now();

                let transfer_copy = transfer.clone();
                let update_type = self.get_job_type();
                let progress = self.progress.clone();
                task::spawn(async move {
                    let msg = &format!(
                        "progress::{}::{}/{}/{}",
                        update_type,
                        progress.items_added,
                        progress.items_total,
                        progress.files_added,
                    );
                    FilesApi::new()
                        .send_upload_status_update_async(&transfer_copy, msg)
                        .await;
                });
            }

            Ok(())
        }
    }
}
