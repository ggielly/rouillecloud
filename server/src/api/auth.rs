//! Authentication API endpoints

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    token: Option<String>,
    message: String,
}

pub async fn login(
    req: web::Json<LoginRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    // TODO: Use plugin manager to authenticate
    let authenticated = req.username == "admin" && req.password == "password";
    
    if authenticated {
        // TODO: Generate JWT token
        let token = "dummy_jwt_token".to_string();
        Ok(HttpResponse::Ok().json(LoginResponse {
            success: true,
            token: Some(token),
            message: "Login successful".to_string(),
        }))
    } else {
        Ok(HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            token: None,
            message: "Invalid credentials".to_string(),
        }))
    }
}

pub async fn logout() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/logout", web::post().to(logout));
}
