# QR Code Service

QR Code Service is a microservice built with Rust and Axum.
It accepts text, generates a QR code in PNG or SVG format, and returns the generated file as a Base64-encoded string.

## What This Service Does

- Exposes one HTTP endpoint: `POST /GetQrCode`
- Accepts input text and output format (`png` or `svg`)
- Returns JSON with:
	- selected format
	- generated file encoded as Base64
- Supports optional API key authentication via header `x-api-key`
- Allows runtime configuration through CLI arguments

## Tech Stack

- Rust
- Axum
- Tokio
- qrcode
- image
- base64
- clap

## Requirements

- Rust toolchain installed (stable recommended)
- Cargo

Check versions:

```bash
rustc --version
cargo --version
```

## Build

Debug build:

```bash
cargo build
```

Release build:

```bash
cargo build --release
```

## Run

### 1. Run with API key enabled

```bash
cargo run -- --bind-addr 0.0.0.0:5020 --api-key secret123
```

### 2. Run without API key

If `--api-key` is not provided (or empty), authentication is disabled.

```bash
cargo run -- --bind-addr 0.0.0.0:5020
```

## Run as Windows Service

Use a release build before creating a service:

```powershell
cargo build --release
```

Executable path:

```text
target\release\qrcode_service.exe
```

### Option A (recommended): NSSM

Install NSSM, then run PowerShell as Administrator:

```powershell
nssm install QrCodeService "D:\Projekt\Rust\QrcodeService\qrcode_service\target\release\qrcode_service.exe" --bind-addr 0.0.0.0:5020 --api-key secret123
nssm set QrCodeService AppDirectory "D:\Projekt\Rust\QrcodeService\qrcode_service"
nssm set QrCodeService AppStdout "D:\Projekt\Rust\QrcodeService\qrcode_service\logs\service.out.log"
nssm set QrCodeService AppStderr "D:\Projekt\Rust\QrcodeService\qrcode_service\logs\service.err.log"
nssm set QrCodeService Start SERVICE_AUTO_START
nssm start QrCodeService
```

### Option B: Native Windows Service (without NSSM)

Run PowerShell as Administrator:

```powershell
New-Service -Name "QrCodeService" `
	-BinaryPathName '"D:\Projekt\Rust\QrcodeService\qrcode_service\target\release\qrcode_service.exe" --bind-addr 0.0.0.0:5020 --api-key secret123' `
	-DisplayName "QR Code Service" `
	-StartupType Automatic

Start-Service QrCodeService
```

Set restart-on-failure policy:

```powershell
sc.exe failure QrCodeService reset= 86400 actions= restart/5000/restart/5000/restart/5000
```

### Service Management

```powershell
Get-Service QrCodeService
Restart-Service QrCodeService
Stop-Service QrCodeService
sc.exe delete QrCodeService
```

Security note:

- API key in service arguments can be visible to system administrators.
- For higher security, prefer storing secrets in environment variables or an external secret manager.

## CLI Arguments

| Argument | Required | Default | Description |
|---|---|---|---|
| `--bind-addr` | No | `0.0.0.0:5020` | Address and port for the HTTP server |
| `--api-key` | No | none | API key for request authorization |

Show CLI help:

```bash
cargo run -- --help
```

## API

### Endpoint

- Method: `POST`
- Path: `/GetQrCode`
- Content-Type: `application/json`

### Authentication

- If service is started with `--api-key <value>`, each request must include header:
	- `x-api-key: <value>`
- If service is started without `--api-key`, authentication is not required.

### Request Body

```json
{
	"text": "https://example.com",
	"format": "png"
}
```

Fields:

- `text` (string, required): source text to encode in QR
- `format` (string, required): one of:
	- `png`
	- `svg`

### Success Response

Status: `200 OK`

```json
{
	"format": "png",
	"file_base64": "iVBORw0KGgoAAAANSUhEUgAA..."
}
```

Fields:

- `format`: resulting file format (`png` or `svg`)
- `file_base64`: generated file encoded as Base64

## Error Responses

- `400 Bad Request`
	- empty `text`
	- invalid QR input
- `401 Unauthorized`
	- missing or invalid `x-api-key` when API key auth is enabled
- `500 Internal Server Error`
	- PNG rendering/encoding failure

## Request Examples

### Curl (Linux/macOS/Git Bash)

With API key:

```bash
curl -X POST "http://localhost:5020/GetQrCode" \
	-H "Content-Type: application/json" \
	-H "x-api-key: secret123" \
	-d '{"text":"Hello, world!","format":"svg"}'
```

Without API key:

```bash
curl -X POST "http://localhost:5020/GetQrCode" \
	-H "Content-Type: application/json" \
	-d '{"text":"Hello, world!","format":"png"}'
```

### PowerShell (Windows)

With API key:

```powershell
Invoke-RestMethod -Method POST -Uri "http://localhost:5020/GetQrCode" `
	-Headers @{ "x-api-key" = "secret123" } `
	-ContentType "application/json" `
	-Body '{"text":"Hello, world!","format":"png"}'
```

Without API key:

```powershell
Invoke-RestMethod -Method POST -Uri "http://localhost:5020/GetQrCode" `
	-ContentType "application/json" `
	-Body '{"text":"Hello, world!","format":"svg"}'
```

## Tests

Run all tests:

```bash
cargo test
```

Run one test:

```bash
cargo test tests::test_get_qr_code_svg -- --exact --nocapture
```

## Project Structure

```text
.
├─ Cargo.toml
├─ README.md
└─ src/
	 └─ main.rs
```

## Notes

- Stopping the server with Ctrl+C is expected and normal.
- Base64 payload can be large for higher image dimensions.



