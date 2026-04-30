# Authenticode signing helper for SecureImage Forge.
#
# Inputs (env or parameters):
#   $env:CERT_PATH   PFX file path
#   $env:CERT_PASS   PFX password
#   $env:TIMESTAMP   RFC 3161 timestamp authority (default: DigiCert)
#
# Usage: .\sign.ps1 -Target "SecureImageForge-0.1.0.exe"
param(
    [Parameter(Mandatory=$true)] [string] $Target
)

$ErrorActionPreference = "Stop"

if (-not $env:CERT_PATH) { throw "CERT_PATH env var is required" }
if (-not $env:CERT_PASS) { throw "CERT_PASS env var is required" }
$tsa = if ($env:TIMESTAMP) { $env:TIMESTAMP } else { "http://timestamp.digicert.com" }

# Use the bundled signtool from the Windows SDK.
$signtool = (Get-Command "signtool.exe").Source

& $signtool sign `
    /fd sha256 `
    /tr $tsa `
    /td sha256 `
    /f "$env:CERT_PATH" `
    /p "$env:CERT_PASS" `
    "$Target"

& $signtool verify /pa /v "$Target"
