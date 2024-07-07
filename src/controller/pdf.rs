use actix::{Addr, StreamHandler};
use actix_http::ws;
use tempfile::NamedTempFile;
use actix_web_actors::ws::Message;
use actix_web::{web, Error as ActixError, HttpResponse};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use log::{info, error, debug};
use zip::write::FileOptions;
use zip::ZipWriter;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use std::io;
use std::path::Path;
use std::fs;
use std::io::Cursor;

use crate::models::pdf::CompressionLevel;
use crate::utils::format_file::format_file_size;

pub async fn post_compress_pdf(
    mut payload: Multipart,
) -> Result<HttpResponse, ActixError> {
    info!("Receiving PDF files for compression");
    // let operation_id = uuid::Uuid::new_v4().to_string();
    let operation_id = "compress_pdf_operation";


    let mut compressed_files = Vec::new();
    let mut file_names = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap_or("unnamed.pdf").to_string();
        let mut temp_file = NamedTempFile::new()?;

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            temp_file.write_all(&data)?;
        }

        let original_size = temp_file.path().metadata()?.len();
        info!("PDF file '{}' received and saved temporarily (size: {})", filename, format_file_size(original_size));

        let compressed_content = compress_pdf(temp_file.path(), CompressionLevel::Low)?;
        let compressed_size = compressed_content.len() as u64;
        let compression_ratio = (compressed_size as f64 / original_size as f64) * 100.0;
        let size_reduction = 100.0 - compression_ratio;
        
        debug!("File '{}' compressed: {} -> {} (reduced by {:.2}%)", 
               filename, 
               format_file_size(original_size), 
               format_file_size(compressed_size), 
               size_reduction);

        compressed_files.push(compressed_content);
        file_names.push(filename);


    }

    let result = match compressed_files.len() {
        0 => Err(ActixError::from(io::Error::new(io::ErrorKind::InvalidInput, "No files were uploaded"))),
        1 => {
            let file_size = format_file_size(compressed_files[0].len() as u64);
            info!("Sending single compressed PDF file (size: {})", file_size);
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .body(compressed_files.pop().unwrap()))
        },
        n => {
            info!("Sending {} compressed PDF files as ZIP", n);
            let zip_content = create_zip(compressed_files, file_names)?;
            let zip_size = format_file_size(zip_content.len() as u64);
            info!("ZIP file created (size: {})", zip_size);
            Ok(HttpResponse::Ok()
                .content_type("application/zip")
                .body(zip_content))
        }
    };

    result
}

pub fn compress_pdf(input_path: &Path, compression_level: CompressionLevel) -> io::Result<Vec<u8>> {
    let input_content = fs::read(input_path)?;
    let input_size = input_content.len() as u64;
    info!("Input file size: {}", format_file_size(input_size));
    if !input_content.starts_with(b"%PDF") {
        error!("Input file does not appear to be a valid PDF");
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid PDF file"));
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
