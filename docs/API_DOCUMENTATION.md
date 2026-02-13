# Running Chronometer API (Rust)

Une API Rust pour une application mobile de chronomètre de course à pied avec support des capteurs, défis, et classements.

## Architecture

### Modèles de données

- **Users**: Utilisateurs avec authentification par mot de passe
- **Routes**: Parcours de course créés par les utilisateurs (public/privé)
- **Scores**: Temps et métriques pour chaque parcours complété
- **SensorData**: Données détaillées des capteurs (accéléromètre, gyroscope, etc.)
- **Challenges**: Défis entre utilisateurs
- **Friendships**: Relations d'amitié entre utilisateurs

## Endpoints API

### Authentication

```
POST   /auth/register      # Créer un compte
POST   /auth/login         # Se connecter
```

### Routes/Parcours

```
POST   /routes             # Créer un parcours
GET    /routes             # Lister tous les parcours (avec filtres)
GET    /routes/:id         # Récupérer un parcours
PUT    /routes/:id         # Mettre à jour un parcours
DELETE /routes/:id         # Supprimer un parcours
GET    /routes/user/:user_id   # Parcours d'un utilisateur
GET    /routes/public      # Parcours publics
POST   /routes/:id/score   # Soumettre un temps/score
```

### Scores & Leaderboard

```
GET    /api/scores/:score_id              # Détails d'un score
GET    /api/leaderboard/route/:route_id   # Classement pour un parcours
GET    /api/leaderboard/global/speed      # Top vitesses globales
```

### Amis

```
POST   /friends/add/:friend_id      # Ajouter un ami (demande en attente)
GET    /friends                      # Lister les amis acceptés
GET    /friends/pending              # Demandes en attente
PUT    /friends/accept/:friendship_id    # Accepter une demande
PUT    /friends/reject/:friendship_id    # Rejeter une demande
```

### Défis

```
POST   /api/challenges                     # Créer un défi
GET    /api/challenges/:id                 # Détails d'un défi
POST   /api/challenges/:id/accept          # Accepter un défi
POST   /api/challenges/:id/complete        # Terminer un défi
GET    /api/challenges/available           # Défis ouverts disponibles
```

### Données de capteurs

```
POST   /sensor-data/:score_id              # Upload un point de données
POST   /sensor-data/bulk                   # Upload en masse
GET    /sensor-data/score/:score_id        # Récupérer données capteur
```

## Base de données

### Migrations appliquées

1. `20260202233909_create_users_table.sql` - Table users
2. `20260202234106_create_posts_table.sql` - Table posts
3. `20260212014820_add_password_to_users.sql` - Colonne password
4. `20260213182953_friend_col.sql` - Migration amis initiale
5. `20260213190000_create_friendships_table.sql` - Table friendships
6. `20260213190100_create_routes_table.sql` - Table routes
7. `20260213190200_create_scores_table.sql` - Table scores
8. `20260213190300_create_challenges_table.sql` - Table challenges
9. `20260213190400_create_sensor_data_table.sql` - Table sensor_data

### Schéma des données

#### routes
```sql
id, user_id, name, description, is_public, path_data (JSONB), 
distance_meters, created_at, updated_at
```

#### scores
```sql
id, route_id, user_id, time_seconds, max_speed_kmh, avg_speed_kmh,
max_g_force, max_inclination_degrees, max_sound_db, created_at
```

#### sensor_data
```sql
id, score_id, timestamp_offset_ms,
accel_x, accel_y, accel_z,
gyro_x, gyro_y, gyro_z,
orientation_azimuth, orientation_pitch, orientation_roll,
speed_kmh, g_force, inclination_degrees, sound_db, nearby_devices,
latitude, longitude, altitude
```

#### challenges
```sql
id, route_id, challenger_id, challenged_id (nullable),
status (pending|active|completed|cancelled), challenger_time, challenged_time,
winner_id, created_at, completed_at
```

#### friendships
```sql
id, user_id, friend_id, status (pending|accepted|rejected), created_at
```

## Installation

### Prérequis

- Rust 1.70+
- PostgreSQL 12+
- Docker (optionnel, pour la base de données)

### Setup local

1. Cloner le repo
```bash
git clone <repo-url>
cd rust-rmce-api
```

2. Configurer l'environnement
```bash
cp .env.example .env
# Éditer .env avec vos configuration PostgreSQL
```

3. Créer la base de données
```bash
createdb rust-rmce-api
```

4. Appliquer les migrations
```bash
sqlx migrate run
```

5. Compiler et lancer
```bash
cargo build
cargo run
```

### Docker Compose

```bash
docker-compose up -d
cargo run
```

## Sécurité

### TODO: JWT Authentication

Les routes suivantes doivent être protégées par JWT:
- Toutes les routes `/routes`, `/friends`, `/api/challenges`, `/sensor-data`
- Le token JWT doit être passé dans l'header `Authorization: Bearer <token>`

**Implémentation prévue:**
- Utiliser la crate `jsonwebtoken`
- Middleware d'extraction du JWT dans les headers
- Claims JWT: `{user_id, exp, iat}`
- Stockage du token côté client (Flutter): SharedPreferences

### Stockage des tokens (Flutter)

**Côté client Android/Flutter:**
```dart
// Stocker le JWT dans SharedPreferences après login
final prefs = await SharedPreferences.getInstance();
await prefs.setString('jwt_token', token);

// Récupérer et utiliser le token dans les requêtes
final token = prefs.getString('jwt_token');
headers['Authorization'] = 'Bearer $token';
```

**NE PAS:**
- Stocker les tokens en dur dans le code
- Utiliser localStorage (comme sur web)
- Passer les tokens dans les query parameters

**À FAIRE:**
- Utiliser `flutter_secure_storage` pour plus de sécurité
- Implémenter refresh token mechanism
- Révoquer les tokens à la déconnexion

### Bonnes pratiques

1. **Validation des entrées**: Utiliser les types Rust pour forcer la validation
2. **Rate limiting**: À implémenter avec middleware
3. **CORS**: À configurer correctement pour la frontend Flutter
4. **HTTPS**: Toujours en production
5. **Passwords**: Hachés avec bcrypt (déjà implémenté)

## Dépendances principales

```toml
axum = "0.8.8"              # Framework web
bcrypt = "0.14"             # Hachage des mots de passe
chrono = { version = "0.4", features = ["serde"] }  # Dates
jsonwebtoken = "9"          # JWT
sqlx = "0.8.6"              # ORM/Query builder
tokio = "1.49.0"            # Runtime async
serde = "1.0.228"           # Sérialisation
uuid = "1.0"                # UUIDs
```

## TODO

- [ ] Implémenter JWT authentication middleware
- [ ] Ajouter CORS configuration
- [ ] Implémenter rate limiting
- [ ] Ajouter tests unitaires
- [ ] WebSocket pour défis temps réel (`/ws/challenge/:id`)
- [ ] Validation des données capteur
- [ ] Compression des données capteur
- [ ] Cache Redis pour leaderboards
- [ ] Pagination pour les listes

## Tests

```bash
# Lancer les tests
cargo test

# Avec logs
RUST_LOG=info cargo test -- --nocapture
```

## Déploiement

À définir avec les détails de deployment (Heroku, Railway, Render, etc.)


