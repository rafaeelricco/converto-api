use crate::ws::*;
use crate::utils::*;
use crate::compress::websocket::*;

use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use log::{info, error, debug};

use actix::Addr;
use serde::Serialize;
use serde_json::json;
use tempfile::NamedTempFile;
use actix_multipart::Multipart;
use uuid::Uuid;
use zip::write::FileOptions;
use zip::ZipWriter;
use std::io::Write;
use std::process::Command;
use std::io;
use std::path::Path;
use std::fs;
use std::io::Cursor;
use std::vec;


// post_compress_pdf
// ------------------------------------------------------------------
// TODO: implement a correct way to update ws progress after receive all files. We need a status to informate "All files received, starting compression"
// TODO: implement a correct way to send the file to the user. It's possible send a link to download the file?
pub async fn post_compress_pdf(
    req: HttpRequest,
    mut payload: Multipart,
    file_processor_addr: web::Data<Addr<FileProcessor>>,
) -> Result<HttpResponse, ActixError> {
    let id = Uuid::parse_str(&req.query_string().replace("id=", "")).unwrap();
    let mut files: Vec<FileProgress> = Vec::new();
    let mut temp_files = Vec::new();
    let mut file_names = Vec::new();

    // Initial progress update
    file_processor_addr.send(UpdateProgress { 
        id,
        files: vec![FileProgress { 
            id: Uuid::new_v4().to_string(), 
            progress: 0.0, 
            message: "Converto está recebendo seus arquivos.".to_string(), 
            file_name: None, 
            compression_level: Some("Medium".to_string()) 
        }],
        status: Status::InProgress,
    }).await.unwrap();

    // Receive all files first
    while let Ok(Some(mut field)) = payload.try_next().await {
        let file_id = Uuid::new_v4().to_string();
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap_or("unnamed.pdf").to_string();
        let mut temp_file = NamedTempFile::new()?;

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            temp_file.write_all(&data)?;
        }

        let original_size = temp_file.path().metadata()?.len();
        info!("PDF file '{}' received and saved temporarily (size: {})", filename, format_file_size(original_size));

        temp_files.push((temp_file, file_id.clone()));
        file_names.push(filename.clone());

        files.push(FileProgress { 
            id: file_id, 
            progress: 50.0, 
            file_name: Some(filename.clone()),
            message: format!("Arquivo {} recebido.", filename),
            compression_level: Some("Medium".to_string()), 
        });
    }

    if temp_files.is_empty() {
        return Err(ActixError::from(io::Error::new(io::ErrorKind::InvalidInput, json!({"error": "Nenhum arquivo recebido."}).to_string())));
    }

    // Update progress after receiving all files
    file_processor_addr.send(UpdateProgress { 
        id,
        files: files.clone(),
        status: Status::InProgress,
    }).await.unwrap();

    // Compress all files
    let mut compressed_files = Vec::new();
    for (index, (temp_file, _)) in temp_files.iter().enumerate() {
        let filename = &file_names[index];
        
        let compressed_content = compress_pdf(temp_file.path(), CompressionLevel::Low)?;
        compressed_files.push(compressed_content);

        files[index].progress = 100.0;
        files[index].message = format!("Arquivo {} processado.", filename);
    }

    // Final progress update
    file_processor_addr.send(UpdateProgress { 
        id,
        files: files.clone(),
        status: Status::InProgress,
    }).await.unwrap();

    let result = if compressed_files.len() == 1 {
        file_processor_addr.send(CompleteProcess { id, files, status: Status::Completed }).await.unwrap();
        Ok(HttpResponse::Ok()
            .content_type("application/pdf")
            .body(compressed_files.pop().unwrap()))
    } else {
        let zip_content = create_zip(compressed_files, file_names)?;
        file_processor_addr.send(CompleteProcess { id, files, status: Status::Completed }).await.unwrap();
        Ok(HttpResponse::Ok()
            .content_type("application/zip")
            .body(zip_content))
    };

    result
}

async fn update_progress(file_processor_addr: &web::Data<Addr<FileProcessor>>, id: &Uuid, file_id: &str, filename: Option<String>, progress: f32, message: &str, compression_level: Option<String>) {
    file_processor_addr.send(UpdateProgress { 
        id: *id,
        files: vec![FileProgress { 
            id: file_id.to_string(), 
            progress, 
            file_name: filename,
            message: message.to_string(),
            compression_level: Some(format!("{:?}", compression_level)), 
        }],
        status: Status::InProgress,
    }).await.unwrap();
}

pub fn compress_pdf(input_path: &Path, compression_level: CompressionLevel) -> io::Result<Vec<u8>> {
    let input_content = fs::read(input_path)?;
    let input_size = input_content.len() as u64;
    info!("Input file size: {}", format_file_size(input_size));
    if !input_content.starts_with(b"%PDF") {
        info!("Input file does not appear to be a valid PDF, skipping compression.");
        return Ok(Vec::new()); 
    }

    let output_file = NamedTempFile::new()?;
    let output_path = output_file.path();

    let compression_settings = match compression_level {
        CompressionLevel::Low => "-dPDFSETTINGS=/screen -dColorImageResolution=72 -dGrayImageResolution=72 -dMonoImageResolution=72",
        CompressionLevel::Medium => "-dPDFSETTINGS=/ebook -dColorImageResolution=150 -dGrayImageResolution=150 -dMonoImageResolution=150",
        CompressionLevel::High => "-dPDFSETTINGS=/printer -dColorImageResolution=300 -dGrayImageResolution=300 -dMonoImageResolution=300",
    };

    let gs_cmd = Command::new("gswin64c")
        .args(&[
            "-sDEVICE=pdfwrite",
            "-dCompatibilityLevel=1.4",
            compression_settings,
            "-dNOPAUSE",
            "-dQUIET",
            "-dBATCH",
            &format!("-sOutputFile={}", output_path.to_string_lossy()),
            &input_path.to_string_lossy(),
        ])
        .output()?;

    let command = format!(
        "gswin64c -sDEVICE=pdfwrite -dCompatibilityLevel=1.4 -dPDFSETTINGS=/ebook -dNOPAUSE -dQUIET -dBATCH -sOutputFile={} {}",
        output_path.to_string_lossy(),
        input_path.to_string_lossy()
    );
    info!("Executing command: {}", command);

    if !gs_cmd.status.success() {
        error!("Ghostscript command failed: {}", String::from_utf8_lossy(&gs_cmd.stderr));
        return Err(io::Error::new(io::ErrorKind::Other, "Ghostscript command failed"));
    }

    let compressed_content = fs::read(output_path)?;
    let compressed_size = compressed_content.len() as u64;
    let compression_ratio = (compressed_size as f64 / input_size as f64) * 100.0;
    let size_reduction = 100.0 - compression_ratio;
    info!("Compressed file size: {} (reduced by {:.2}%)", format_file_size(compressed_size), size_reduction);

    Ok(compressed_content)
}

fn create_zip(files: Vec<Vec<u8>>, file_names: Vec<String>) -> io::Result<Vec<u8>> {
    let mut zip_buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buffer));
        let options = FileOptions::<'_, ()>::default().compression_method(zip::CompressionMethod::Stored);

        for (content, name) in files.into_iter().zip(file_names.into_iter()) {
            zip.start_file(name.clone(), options)?;
            zip.write_all(&content)?;
            debug!("Added file '{}' to ZIP (size: {})", name, format_file_size(content.len() as u64));
        }

        zip.finish()?;
    }
    let zip_size = zip_buffer.len() as u64;
    info!("ZIP file created (size: {})", format_file_size(zip_size));
    Ok(zip_buffer)
}

#[derive(Debug, Serialize, Clone)]
pub enum CompressionLevel {
    Low,
    Medium,
    High,
}

impl From<&str> for CompressionLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => CompressionLevel::Low,
            "high" => CompressionLevel::High,
            _ => CompressionLevel::Medium,
        }
    }
}
