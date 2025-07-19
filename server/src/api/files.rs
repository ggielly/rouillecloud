//! File operations API endpoints

use actix_web::{web, HttpResponse, Result, HttpRequest};
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateDirRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct CreateFileRequest {
    name: String,
    content: Option<String>,
}

#[derive(Deserialize)]
pub struct RenameRequest {
    old_name: String,
    new_name: String,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    name: String,
}

#[derive(Serialize)]
pub struct FileItem {
    name: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<String>,
}

pub async fn create_directory(
    req: web::Json<CreateDirRequest>,
    _data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let path = PathBuf::from(&req.name);
    
    match fs::create_dir_all(&path).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": true}))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({"error": format!("Failed to create directory: {}", e)})
        )),
    }
}

pub async fn create_file(
    req: web::Json<CreateFileRequest>,
    _data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let path = PathBuf::from(&req.name);
    let content = req.content.as_deref().unwrap_or("");
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    
    match fs::write(&path, content).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": true}))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({"error": format!("Failed to create file: {}", e)})
        )),
    }
}

pub async fn rename_item(
    req: web::Json<RenameRequest>,
    _data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let old_path = PathBuf::from(&req.old_name);
    let new_path = PathBuf::from(&req.new_name);
    
    match fs::rename(&old_path, &new_path).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": true}))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({"error": format!("Failed to rename: {}", e)})
        )),
    }
}

pub async fn delete_item(
    req: web::Json<DeleteRequest>,
    _data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let path = PathBuf::from(&req.name);
    
    let result = if path.is_dir() {
        fs::remove_dir_all(&path).await
    } else {
        fs::remove_file(&path).await
    };
    
    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({"success": true}))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({"error": format!("Failed to delete: {}", e)})
        )),
    }
}

pub async fn list_directory(
    req: HttpRequest,
    _data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let path = req.query_string()
        .split('&')
        .find(|s| s.starts_with("path="))
        .map(|s| &s[5..])
        .unwrap_or("/");
    
    let path = PathBuf::from(path);
    
    match fs::read_dir(&path).await {
        Ok(mut entries) => {
            let mut items = Vec::new();
            
            while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                let metadata = entry.metadata().await.ok();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().and_then(|m| if !is_dir { Some(m.len()) } else { None });
                
                items.push(FileItem {
                    name: entry.file_name().to_string_lossy().to_string(),
                    is_dir,
                    size,
                    modified: None, // TODO: Add timestamp formatting
                });
            }
            
            Ok(HttpResponse::Ok().json(items))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({"error": format!("Failed to list directory: {}", e)})
        )),
    }
}

pub async fn upload_file(
    mut payload: Multipart,
    _data: web::Data<AppState>,
) -> Result<HttpResponse> {
    use futures::StreamExt;
    
    while let Some(item) = payload.next().await {
        let mut field = item?;
        
        if let Some(filename) = field.content_disposition().get_filename() {
            let filepath = PathBuf::from(filename);
            
            // Create parent directories if they don't exist
            if let Some(parent) = filepath.parent() {
                let _ = fs::create_dir_all(parent).await;
            }
            
            let mut file_data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk?;
                file_data.extend_from_slice(&chunk);
            }
            
            match fs::write(&filepath, &file_data).await {
                Ok(_) => return Ok(HttpResponse::Ok().json(serde_json::json!({"success": true}))),
                Err(e) => return Ok(HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": format!("Failed to save file: {}", e)})
                )),
            }
        }
    }
    
    Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "No file found"})))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/dir", web::post().to(create_directory))
        .route("/file", web::post().to(create_file))
        .route("/rename", web::post().to(rename_item))
        .route("/delete", web::post().to(delete_item))
        .route("/list", web::get().to(list_directory))
        .route("/upload", web::post().to(upload_file));
}
