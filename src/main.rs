use std::env;
use std::fs;
use std::io;
use std::path::Path;
use serde_json::json;
use mime_guess;
use std::time::{SystemTime, UNIX_EPOCH};
use rexiv2::Metadata; // Add rexiv2 for EXIF data extraction

fn main() -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let mut file_data = Vec::new();

    for entry in fs::read_dir(current_dir.clone())? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let metadata = fs::metadata(&path)?;
            let file_size = metadata.len();
            let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let file_extension = path.extension().unwrap_or_default().to_string_lossy().to_string();
            let created_time = metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            let modified_time = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            let accessed_time = metadata
                .accessed()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            // Attempt to extract EXIF data if the file is an image
            let exif_data = if mime_type.type_() == "image" {
                match Metadata::new_from_path(&path) {
                    Ok(meta) => {
                        let mut exif_map = serde_json::Map::new();
                        if let Ok(tags) = meta.get_exif_tags() {
                            for tag in tags {
                                if let Some((key, value)) = tag.split_once('=') {
                                    exif_map.insert(key.to_string(), json!(value));
                                }
                            }
                        }
                        Some(exif_map)
                    }
                    Err(_) => None,
                }
            } else {
                None
            };

            file_data.push(
                json!({
                "file_name": file_name,
                "file_size": file_size,
                "mime_type": mime_type.to_string(),
                "file_extension": file_extension,
                "path": path.to_string_lossy().to_string(),
                "created_time": created_time,
                "modified_time": modified_time,
                "accessed_time": accessed_time,
                "exif_data": exif_data
            })
            );
        }
    }

    let output_file = current_dir.clone().join("file_data.json");
    fs::write(output_file, serde_json::to_string_pretty(&file_data)?)?;

    println!("File data has been written to file_data.json");
    Ok(())
}
