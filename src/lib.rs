use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use rawzip::{ZipArchive, time::ZipDateTimeKind};
use serde::Serialize;
use std::io::{self, BufWriter, Read, Write};

#[derive(Serialize, Debug)]
pub struct Metadata {
    #[serde(rename = "ZipName")]
    pub zip_name: String,
}

#[derive(Serialize, Debug)]
pub struct Blob {
    pub name: String,
    pub content_type: String,
    pub content_encoding: String,
    pub content_transfer_encoding: String,
    pub body: String,
    pub metadata: Metadata,
    pub content_length: u64,
    pub last_modified: String,
}

fn zip_datetime_to_chrono_utc(zdt: &ZipDateTimeKind) -> DateTime<Utc> {
    let (year, month, day, hour, minute, second) = (
        zdt.year(),
        zdt.month(),
        zdt.day(),
        zdt.hour(),
        zdt.minute(),
        zdt.second(),
    );
    let naive_date =
        NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32).unwrap_or_default();
    let naive_time = chrono::NaiveTime::from_hms_opt(hour as u32, minute as u32, second as u32)
        .unwrap_or_default();
    let naive_dt = NaiveDateTime::new(naive_date, naive_time);
    DateTime::from_naive_utc_and_offset(naive_dt, Utc)
}

pub fn stdin2zip2blobs2jsons2stdout(
    zip_name: &str,
    content_type: &str,
    content_encoding: &str,
    max_zip_size: u64,
    max_item_size: u64,
    verbose: bool,
) -> Result<(), io::Error> {
    let stdin_lock = io::stdin().lock();
    let mut buffer = Vec::new();
    // Read up to one byte more than the limit to detect if the file is too large.
    stdin_lock.take(max_zip_size + 1).read_to_end(&mut buffer)?;

    if buffer.len() as u64 > max_zip_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Input zip file exceeds the maximum allowed size of {} bytes.",
                max_zip_size
            ),
        ));
    }

    let archive = ZipArchive::from_slice(&buffer).map_err(io::Error::other)?;

    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    {
        let mut writer = BufWriter::new(&mut stdout_lock);

        for entry_result in archive.entries() {
            let entry_header = entry_result.map_err(io::Error::other)?;
            let wayfinder = entry_header.wayfinder();
            let entry = archive.get_entry(wayfinder).map_err(io::Error::other)?;
            let entry_data = entry.data();
            let file_name =
                String::from_utf8_lossy(entry_header.file_path().as_bytes()).to_string();

            if entry_data.len() as u64 > max_item_size {
                if verbose {
                    eprintln!(
                        "Warning: Skipping item '{}' because its size ({} bytes) exceeds the maximum allowed ({} bytes).",
                        file_name,
                        entry_data.len(),
                        max_item_size
                    );
                }
                continue;
            }

            let dt: DateTime<Utc> = zip_datetime_to_chrono_utc(&entry_header.last_modified());

            let blob = Blob {
                name: file_name,
                content_type: content_type.to_string(),
                content_encoding: content_encoding.to_string(),
                content_transfer_encoding: "base64".to_string(),
                body: general_purpose::STANDARD.encode(entry_data),
                metadata: Metadata {
                    zip_name: zip_name.to_string(),
                },
                content_length: entry_data.len() as u64,
                last_modified: dt.to_rfc3339(),
            };

            serde_json::to_writer(&mut writer, &blob)?;
            writeln!(&mut writer)?;
        }

        writer.flush()?;
    }

    stdout_lock.flush()
}
