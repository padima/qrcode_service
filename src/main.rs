use axum::{
    Router,
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use clap::Parser;
use image::{DynamicImage, ImageFormat, Luma};
use qrcode::{QrCode, render::svg};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Parser, Debug)]
#[command(name = "qrcode_service")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:5020")]
    bind_addr: String,

    #[arg(long)]
    api_key: Option<String>,
}

#[derive(Clone)]
struct AppState {
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QrRequest {
    text: String,
    format: QrFormat,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum QrFormat {
    Png,
    Svg,
}

#[derive(Debug, Serialize)]
struct QrResponse {
    format: &'static str,
    file_base64: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let api_key = cli.api_key.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    let state = AppState { api_key };

    let app = Router::new()
        .route("/GetQrCode", post(get_qr_code))
        .route("/health", post(|| async { "OK" }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cli.bind_addr)
        .await
        .expect("bind failed");

    println!("Listening on {}", cli.bind_addr);

    axum::serve(listener, app).await.expect("server failed");
}

async fn get_qr_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<QrRequest>,
) -> Result<Json<QrResponse>, (StatusCode, String)> {
    if let Some(expected_key) = &state.api_key {
        let provided_key = headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok());

        if provided_key != Some(expected_key.as_str()) {
            return Err((StatusCode::UNAUTHORIZED, "invalid api key".to_string()));
        }
    }

    if payload.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text is empty".to_string()));
    }

    let code = QrCode::new(payload.text.as_bytes())
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    match payload.format {
        QrFormat::Svg => {
            let svg_data = code.render::<svg::Color>().min_dimensions(256, 256).build();

            Ok(Json(QrResponse {
                format: "svg",
                file_base64: STANDARD.encode(svg_data.as_bytes()),
            }))
        }
        QrFormat::Png => {
            let image = code.render::<Luma<u8>>().min_dimensions(256, 256).build();

            let dynamic = DynamicImage::ImageLuma8(image);
            let mut buffer = Cursor::new(Vec::new());

            dynamic
                .write_to(&mut buffer, ImageFormat::Png)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok(Json(QrResponse {
                format: "png",
                file_base64: STANDARD.encode(buffer.into_inner()),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_qr_code_svg() {
        let state = AppState { api_key: None };
        let response = get_qr_code(
            State(state),
            HeaderMap::new(),
            Json(QrRequest {
                text: "Hello, world!".to_string(),
                format: QrFormat::Svg,
            }),
        )
        .await;
        assert!(response.is_ok());
    }
    #[tokio::test]
    async fn test_get_qr_code_png() {
        let state = AppState { api_key: None };
        let response = get_qr_code(
            State(state),
            HeaderMap::new(),
            Json(QrRequest {
                text: "Hello, world!".to_string(),
                format: QrFormat::Png,
            }),
        )
        .await;
        assert!(response.is_ok());
    }
}
