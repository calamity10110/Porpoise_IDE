# Generate a self-signed code signing certificate for Porpoise
# Run this in an Administrator PowerShell session

$cert = New-SelfSignedCertificate `
    -Type Custom `
    -Subject "CN=Porpoise Development,O=Porpoise,CN=Porpoise" `
    -KeyUsage DigitalSignature `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3") `
    -FriendlyName "Porpoise Development Signing" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -NotAfter (Get-Date).AddYears(5)

$password = Read-Host "Enter password for .pfx file" -AsSecureString
$pfxPath = Join-Path (Get-Location) "porpoise-dev.pfx"
Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $password

Write-Host "Certificate created at: $pfxPath" -ForegroundColor Green
Write-Host "Thumbprint: $($cert.Thumbprint)" -ForegroundColor Green

# Set env vars for Tauri to use
Write-Host "`nSet these environment variables for Tauri build:`n" -ForegroundColor Yellow
Write-Host "`$env:TAURI_SIGNING_PRIVATE_KEY = '$pfxPath'`"
Write-Host "`$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '<password>'`"
