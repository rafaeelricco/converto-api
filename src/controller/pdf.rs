use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use futures::{StreamExt, TryStreamExt};

pub async fn compress_pdf(mut payload: Multipart) -> Result<HttpResponse, Error> {
    //! # Compress PDF
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition();
        if let Some(filename) = content_type.get_filename() {
            // Lógica para salvar ou processar o arquivo
        }
    }
    //!
    Ok(HttpResponse::Ok().body("Arquivo recebido e processado"))
}
