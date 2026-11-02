$ErrorActionPreference = 'Stop'

$repo = 'mo33tafa/vidi'
$version = if ($env:VIDI_VERSION) { $env:VIDI_VERSION } else { 'latest' }
$installDir = if ($env:VIDI_INSTALL_DIR) { $env:VIDI_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'vidi\bin' }
$asset = "vidi-windows-x86_64.zip"
$base = "https://github.com/$repo/releases"
$url = if ($version -eq 'latest') {
  "$base/latest/download/$asset"
} else {
  "$base/download/$version/$asset"
}

$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("vidi-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tempDir | Out-Null
try {
  $archive = Join-Path $tempDir $asset
  Write-Host "Downloading vidi for Windows ..."
  Invoke-WebRequest -Uri $url -OutFile $archive
  New-Item -ItemType Directory -Force -Path $installDir | Out-Null
  Expand-Archive -Path $archive -DestinationPath $tempDir -Force
  Copy-Item (Join-Path $tempDir 'vidi.exe') (Join-Path $installDir 'vidi.exe') -Force

  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  $entries = if ($userPath) { $userPath -split ';' } else { @() }
  if ($entries -notcontains $installDir) {
    $newPath = (($entries + $installDir) | Where-Object { $_ }) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
  }

  & (Join-Path $installDir 'vidi.exe') --version
  Write-Host "Installed to $installDir\vidi.exe"
  Write-Host 'Open a new PowerShell window before running vidi.'
} finally {
  Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}
