use axum::{
    Router,
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use clap::Parser;
use image::{DynamicImage, ImageFormat, Luma};
use qrcode::{QrCode, render::svg};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Command-line interface definition.
#[derive(Parser, Debug)]
#[command(name = "qrcode_service")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:5020")]
    bind_addr: String,

    #[arg(long)]
    api_key: Option<String>,
}

/// Application state shared across handlers.
#[derive(Clone)]
struct AppState {
    api_key: Option<String>,
}

/// Request structure for the QR code generation endpoint.
#[derive(Debug, Deserialize)]
struct QrCodeRequest {
    text: String,
    format: QrCodeFormat,
    height: Option<u32>,
    width: Option<u32>,
}

/// Supported QR code formats.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum QrCodeFormat {
    Png,
    Svg,
}

/// Response structure for the QR code generation endpoint.
#[derive(Debug, Serialize)]
struct QrCodeResponse {
    data_base64: String,
}

/// Main entry point of the application.
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
        .route("/health", get(|| async { "OK" }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cli.bind_addr)
        .await
        .expect("bind failed");

    println!("Listening on {}", cli.bind_addr);

    axum::serve(listener, app).await.expect("server failed");
}

/// Handler for generating QR codes.
async fn get_qr_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<QrCodeRequest>,
) -> Result<Json<QrCodeResponse>, (StatusCode, String)> {
    if let Some(expected_key) = &state.api_key {
        let provided_key = headers
            .get("x-api-key")
            .and_then(|value| value.to_str().ok());

        if provided_key != Some(expected_key.as_str()) {
            return Err((StatusCode::UNAUTHORIZED, "invalid api key".to_string()));
        }
    }

    if body.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text is empty".to_string()));
    }

    let height = body.height.unwrap_or(256);
    let width = body.width.unwrap_or(256);

    let code =
        QrCode::new(body.text.as_bytes()).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    match body.format {
        QrCodeFormat::Svg => {
            let svg_data = code
                .render::<svg::Color>()
                .min_dimensions(width, height)
                .build();

            Ok(Json(QrCodeResponse {
                data_base64: STANDARD.encode(svg_data.as_bytes()),
            }))
        }
        QrCodeFormat::Png => {
            let image = code
                .render::<Luma<u8>>()
                .min_dimensions(width, height)
                .build();

            let dynamic = DynamicImage::ImageLuma8(image);
            let mut buffer = Cursor::new(Vec::new());

            dynamic
                .write_to(&mut buffer, ImageFormat::Png)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok(Json(QrCodeResponse {
                data_base64: STANDARD.encode(buffer.into_inner()),
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
            Json(QrCodeRequest {
                text: "Hello, world!".to_string(),
                format: QrCodeFormat::Svg,
                height: Some(256),
                width: Some(256),
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
            Json(QrCodeRequest {
                text: "Hello, world!".to_string(),
                format: QrCodeFormat::Png,
                height: Some(256),
                width: Some(256),
            }),
        )
        .await;
        assert!(response.is_ok());
    }
}
