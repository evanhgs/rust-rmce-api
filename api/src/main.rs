use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing::{info, error};

use rust_rmce_api::{db, routes};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    // Configuration du logging avec filtres d'environnement
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Démarrage de l'application rust-rmce-api");

    let url = std::env::var("DATABASE_URL").map_err(|_| {
        error!("DATABASE_URL n'est pas définie dans les variables d'environnement");
        error!("Créez un fichier .env avec: DATABASE_URL=postgresql://user:password@localhost:5432/dbname");
        sqlx::Error::PoolClosed
    })?;

    info!("Connexion à la base de données...");
    let pool = db::create_pool(&url).await.map_err(|e| {
        error!("Erreur lors de la connexion à la base de données: {}", e);
        error!("Vérifiez que:");
        error!("  1. La base de données PostgreSQL est en cours d'exécution");
        error!("  2. La variable DATABASE_URL est correctement configurée");
        error!("  3. Les identifiants de connexion sont valides");
        error!("  4. Le port PostgreSQL est accessible (vérifiez avec: docker ps)");
        e
    })?;

    info!("Pool de connexions créé avec succès");

    let app = routes::create_app(pool);

    let addr = "0.0.0.0:5000";
    info!("Serveur HTTP démarré sur {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        error!("Impossible de lier le serveur sur {}: {}", addr, e);
        sqlx::Error::PoolClosed
    })?;

    info!("Application prête à recevoir des requêtes");
    axum::serve(listener, app).await.map_err(|e| {
        error!("Erreur du serveur: {}", e);
        sqlx::Error::PoolClosed
    })?;

    Ok(())
}
