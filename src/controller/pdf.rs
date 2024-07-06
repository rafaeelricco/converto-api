use actix_web::{HttpResponse, Responder};


pub async fn compress_pdf() -> impl Responder {
    //! # Compress PDF
    //!
    //! This function will be responsible for compressing a PDF file. It will receive a PDF file and return a compressed PDF file.
    //!
    HttpResponse::Ok().body("Compress PDF")
}