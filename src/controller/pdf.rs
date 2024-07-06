use tempfile::NamedTempFile;
use actix_web::{HttpResponse, Error as ActixError};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use log::{info, error};
use std::io::Write;
use std::process::Command;
use std::io;
use std::path::Path;
use std::fs;
use crate::models::pdf::CompressionLevel;

pub async fn post_compress_pdf(mut payload: Multipart) -> Result<HttpResponse, ActixError> {
    info!("Receiving PDF file for compression");
    let mut temp_file = NamedTempFile::new()?;

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            temp_file.write_all(&data)?;
        }
    }
    info!("PDF file received and saved temporarily");

    let compressed_content = compress_pdf(temp_file.path(), CompressionLevel::Medium)?;
    
    info!("Sending compressed PDF file");
    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .body(compressed_content))
}

pub fn compress_pdf(input_path: &Path, compression_level: CompressionLevel) -> io::Result<Vec<u8>> {
    info!("Compressing PDF file: {:?}", input_path);

    let input_content = fs::read(input_path)?;
    info!("Input file size: {} bytes", input_content.len());
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
    info!("Compressed file size: {} bytes", compressed_content.len());

    Ok(compressed_content)
}
