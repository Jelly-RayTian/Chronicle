param(
  [string]$InstallerGlob = "src-tauri/target/release/bundle/nsis/*.exe",
  [string]$OutputPath = "dist/release/SHA256SUMS.txt"
)

$ErrorActionPreference = "Stop"
$releaseVersion = (Get-Content -LiteralPath "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json).version
$installers = @(
  Get-ChildItem -Path $InstallerGlob -File |
    Where-Object {
      $_.Name.StartsWith("Chronicle_${releaseVersion}_") -and $_.Extension -eq ".exe"
    }
)
if ($installers.Count -ne 1) {
  throw "Expected exactly one v$releaseVersion Windows NSIS installer at $InstallerGlob; found $($installers.Count)."
}

$installer = $installers[0]
if ($installer.Length -lt 1MB) {
  throw "Installer is unexpectedly small: $($installer.Length) bytes."
}

$bytes = [System.IO.File]::ReadAllBytes($installer.FullName)
if ($bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
  throw "Installer does not have a Windows PE header."
}

$signature = Get-AuthenticodeSignature -LiteralPath $installer.FullName
if ($signature.Status -notin @("Valid", "NotSigned")) {
  throw "Installer signature status is $($signature.Status): $($signature.StatusMessage)"
}

$outputDirectory = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
$hash = Get-FileHash -Algorithm SHA256 -LiteralPath $installer.FullName
"$($hash.Hash.ToLowerInvariant())  $($installer.Name)" | Set-Content -LiteralPath $OutputPath -Encoding ascii

Write-Host "Validated $($installer.Name) ($($installer.Length) bytes)."
Write-Host "Authenticode status: $($signature.Status)."
Write-Host "SHA-256: $($hash.Hash.ToLowerInvariant())."
