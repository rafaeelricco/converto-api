use crate::controller::pdf::compress_pdf;
use actix_web::web;

pub fn configure_pdf_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/pdf")
        .route("/compress", web::post().to(compress_pdf))
    );
}
