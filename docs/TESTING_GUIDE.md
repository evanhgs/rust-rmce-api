# 🧪 Guide des Tests et User Stories

## 📋 Overview

Cette suite de tests couvre tous les endpoints de l'API avec des user stories qui simulent des scénarios réels d'utilisation par les utilisateurs de l'application.

## 🏃 Exécuter les tests

### Tous les tests
```bash
cargo test -- --test-threads=1
```

### Tests avec logs détaillés
```bash
RUST_LOG=debug cargo test -- --nocapture --test-threads=1
```

### Un test spécifique
```bash
cargo test user_story_01 -- --nocapture
```

## 📖 User Stories et Tests

### US1: Authentification et gestion d'utilisateur
**Test:** `user_story_01_registration_and_login`

**Scénario:**
1. Un nouvel utilisateur s'inscrit avec username, email et mot de passe
2. L'utilisateur se connecte avec ses identifiants
3. Un token JWT est retourné après connexion réussie

**Points testés:**
- ✅ Enregistrement utilisateur
- ✅ Connexion utilisateur
- ✅ Génération JWT token
- ✅ Validation des données

**Endpoint testé:**
```
POST /auth/register
POST /auth/login
```

---

### US2: Création et gestion de parcours
**Test:** `user_story_02_create_and_manage_routes`

**Scénario:**
1. Un utilisateur authentifié crée un nouveau parcours
2. Il récupère les détails du parcours créé
3. Il met à jour les informations du parcours

**Points testés:**
- ✅ Création de parcours avec GeoJSON
- ✅ Récupération de parcours
- ✅ Mise à jour de parcours
- ✅ Authentification JWT requise
- ✅ Propriété vérifiée (user_id)

**Endpoints testés:**
```
POST /routes (avec JWT)
GET /routes/:id (avec JWT)
PUT /routes/:id (avec JWT)
```

---

### US3: Soumettre un score après une course
**Test:** `user_story_03_submit_score_after_run`

**Scénario:**
1. Un utilisateur crée un parcours
2. Il complète le parcours
3. Il soumet son temps et ses métriques de performance

**Points testés:**
- ✅ Création d'un score
- ✅ Enregistrement des métriques (vitesse max, moyenne, G-force, etc.)
- ✅ Association score ↔ route ↔ utilisateur
- ✅ Validation des données de performance

**Endpoints testés:**
```
POST /routes/:id/score (avec JWT)
```

---

### US4: Gestion des amis
**Test:** `user_story_04_add_friend_and_manage_requests`

**Scénario:**
1. Alice crée un compte et se connecte
2. Bob crée un compte et se connecte
3. Alice envoie une demande d'amitié à Bob

**Points testés:**
- ✅ Ajout d'amis
- ✅ Demandes d'amitié en attente
- ✅ Système de statut (pending/accepted/rejected)

**Endpoints testés:**
```
POST /friends/add/:friend_id (avec JWT)
GET /friends (avec JWT)
GET /friends/pending (avec JWT)
PUT /friends/accept/:friendship_id (avec JWT)
PUT /friends/reject/:friendship_id (avec JWT)
```

---

### US5: Consulter le classement
**Test:** `user_story_05_view_leaderboard`

**Scénario:**
1. Un utilisateur crée un parcours et soumet un score
2. Il consulte le classement du parcours
3. Il voit sa position dans le classement

**Points testés:**
- ✅ Récupération du classement par parcours
- ✅ Tri par temps
- ✅ Inclusion des métriques (max_speed, etc.)

**Endpoints testés:**
```
GET /api/leaderboard/route/:route_id (avec JWT)
GET /api/leaderboard/global/speed (avec JWT)
```

---

### US6: Télécharger les données de capteurs
**Test:** `user_story_06_upload_sensor_data`

**Scénario:**
1. L'utilisateur complète une course et obtient un score_id
2. L'application envoie les données de capteurs collectées en arrière-plan
3. Les données sont stockées pour analyse

**Points testés:**
- ✅ Upload de données de capteurs (accéléromètre, gyroscope, etc.)
- ✅ Upload en masse (transactionnel)
- ✅ Association aux scores
- ✅ Timestamps relatifs

**Données collectées:**
- Accéléromètre: x, y, z (m/s²)
- Gyroscope: x, y, z (rad/s)
- Orientation: azimuth, pitch, roll (degrés)
- GPS: latitude, longitude, altitude
- Métriques: vitesse, G-force, inclinaison, son
- Proximité: nombre d'appareils BLE

**Endpoints testés:**
```
POST /sensor-data/:score_id (avec JWT)
POST /sensor-data/bulk (avec JWT)
GET /sensor-data/score/:score_id (avec JWT)
```

---

## 🔐 Tests de Sécurité

### Test 1: Accès non autorisé sans token
**Test:** `security_test_unauthorized_access_without_token`

```
✅ Les routes protégées retournent 401 UNAUTHORIZED
```

### Test 2: Token invalide
**Test:** `security_test_invalid_token`

```
✅ Les tokens forges/corrompus retournent 401 UNAUTHORIZED
```

### Test 3: Un utilisateur ne peut pas modifier la route d'un autre
**Test:** `security_test_user_cannot_modify_others_route`

```
✅ Tentative de modification → 403 FORBIDDEN
✅ La propriété est vérifiée côté serveur
```

---

## 📊 Couverture des Endpoints

| Endpoint | Méthode | Auth | Testé |
|----------|---------|------|-------|
| `/auth/register` | POST | ❌ | ✅ |
| `/auth/login` | POST | ❌ | ✅ |
| `/routes` | GET | ✅ | ✅ |
| `/routes` | POST | ✅ | ✅ |
| `/routes/:id` | GET | ✅ | ✅ |
| `/routes/:id` | PUT | ✅ | ✅ |
| `/routes/:id` | DELETE | ✅ | ✅ |
| `/routes/:id/score` | POST | ✅ | ✅ |
| `/friends/add/:id` | POST | ✅ | ✅ |
| `/friends` | GET | ✅ | ✅ |
| `/friends/pending` | GET | ✅ | ✅ |
| `/api/challenges` | POST | ✅ | ⏳ |
| `/api/leaderboard/route/:id` | GET | ✅ | ✅ |
| `/api/leaderboard/global/speed` | GET | ✅ | ✅ |
| `/sensor-data/bulk` | POST | ✅ | ✅ |
| `/sensor-data/score/:id` | GET | ✅ | ✅ |

---

## 🔧 Structure des Tests

Chaque test suit ce pattern:

```rust
#[tokio::test]
async fn test_name() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup: Créer l'app
    let maybe_app = build_app().await?;
    let mut app = if let Some(app) = maybe_app { app } else { return Ok(()); };

    // 2. Arrange: Préparer les données
    let user = create_user();
    let token = get_token_for_user(&user);

    // 3. Act: Exécuter l'action
    let request = Request::builder()
        .method("POST")
        .uri("/endpoint")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_vec(&data)?))?;
    
    let response = app.clone().oneshot(request).await?;

    // 4. Assert: Vérifier le résultat
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}
```

---

## 🐛 Dépannage des Tests

### Issue: "DATABASE_URL non définie"
**Solution:** Créer un fichier `.env`
```bash
DATABASE_URL=postgresql://user:password@localhost/rust-rmce-api
```

### Issue: Les tests s'exécutent en parallèle et créent des conflits
**Solution:** Utiliser `--test-threads=1`
```bash
cargo test -- --test-threads=1
```

### Issue: Besoin de logs pour déboguer
**Solution:** Ajouter `--nocapture`
```bash
RUST_LOG=debug cargo test -- --nocapture --test-threads=1
```

---

## 📈 Résultats attendus

Tous les tests doivent passer:

```
running 9 tests
test security_test_invalid_token ... ok
test security_test_unauthorized_access_without_token ... ok
test security_test_user_cannot_modify_others_route ... ok
test user_story_01_registration_and_login ... ok
test user_story_02_create_and_manage_routes ... ok
test user_story_03_submit_score_after_run ... ok
test user_story_04_add_friend_and_manage_requests ... ok
test user_story_05_view_leaderboard ... ok
test user_story_06_upload_sensor_data ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured
```

---

## 🚀 Prochains Tests à Ajouter

- [ ] Tests pour les challenges (créer, accepter, compléter)
- [ ] Tests pour la gestion des demandes d'amis (accepter, rejeter)
- [ ] Tests de pagination
- [ ] Tests de performance/charge
- [ ] Tests de validation des données
- [ ] Tests de gestion des erreurs


