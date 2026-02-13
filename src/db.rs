use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use tracing::{info, error};

pub type DbPool = Pool<Postgres>;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    // Masquer le mot de passe dans les logs pour la sécurité
    let safe_url = mask_password_in_url(database_url);
    info!("Création du pool de connexions à la base de données");
    info!("URL (masquée): {}", safe_url);
    
    // Extraire les informations de connexion pour les logs
    if let Some(host_start) = database_url.find('@') {
        if let Some(host_end) = database_url[host_start..].find(':') {
            let host = &database_url[host_start+1..host_start+host_end];
            if let Some(port_start) = database_url[host_start+host_end..].find(':') {
                if let Some(port_end) = database_url[host_start+host_end+port_start+1..].find('/') {
                    let port = &database_url[host_start+host_end+port_start+1..host_start+host_end+port_start+1+port_end];
                    info!("Host: {}, Port: {}", host, port);
                }
            }
        }
    }
    
    info!("Tentative de connexion...");
    let start = std::time::Instant::now();
    
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await;

    let elapsed = start.elapsed();
    
    match &pool {
        Ok(_) => {
            info!("✓ Pool de connexions créé avec succès en {:?}", elapsed);
            info!("  - Connexions max: 10");
            info!("  - Timeout d'acquisition: 30s");
            info!("  - Timeout d'inactivité: 600s");
            info!("  - Durée de vie max: 1800s");
        }
        Err(e) => {
            error!("✗ Échec de la création du pool de connexions après {:?}", elapsed);
            error!("Erreur: {}", e);
            error!("URL utilisée: {}", safe_url);
            error!("Vérifiez que:");
            error!("  1. PostgreSQL est accessible sur le host/port spécifié");
            error!("  2. Les identifiants sont corrects");
            error!("  3. La base de données existe");
            error!("  4. Le port est bien exposé (docker ps pour vérifier)");
        }
    }

    pool
}

fn mask_password_in_url(url: &str) -> String {
    // Masque le mot de passe dans l'URL pour les logs
    if let Some(at_pos) = url.find('@') {
        if let Some(slash_pos) = url[..at_pos].rfind(':') {
            if let Some(user_pos) = url[..slash_pos].rfind("://") {
                let protocol = &url[..user_pos + 3];
                let user = &url[user_pos + 3..slash_pos];
                let rest = &url[at_pos..];
                return format!("{}{}:***{}", protocol, user, rest);
            }
        }
    }
    url.to_string()
}

