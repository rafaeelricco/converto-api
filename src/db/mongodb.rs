use log::info;
use mongodb::{options::ClientOptions, Client};
use std::sync::Arc;

pub type Pool = Arc<Client>;
// pub type SharedDatabase = Arc<Database>;

pub async fn init_db_pool(url: &str) -> Result<Pool, mongodb::error::Error> {
    info!("Connecting to MongoDB...");

    let mut client_options = ClientOptions::parse(url).await?;
    client_options.app_name = Some("actix-web-mongodb".to_string());

    let client = Client::with_options(client_options)?;
    info!("Connected to MongoDB at {}", url);   

    Ok(Arc::new(client))
}

// pub async fn get_db() -> SharedDatabase {
//     let address = dotenv::var("DB_URL").expect("A variável de ambiente 'DB_URL' não está definida. Por favor, defina-a no seu arquivo .env.");

//     let mut client_options = ClientOptions::parse(&address)
//         .await
//         .expect("Erro ao inicializar as opções do cliente do MongoDB.");
//     client_options.app_name = Some("actix-web-mongodb".to_string());

//     let client =
//         Client::with_options(client_options).expect("Erro ao inicializar o cliente do MongoDB.");
//     let db = client.database("rust-actix-web-mongodb");

//     Arc::new(db)
// }
