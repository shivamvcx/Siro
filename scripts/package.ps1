# Siro - Packaging Script
# Builds release binary (siro.exe & siro_portable.exe), Inno Setup installer (Siro_Setup.exe), and WiX package (Siro.msi)

$ErrorActionPreference = "Stop"

Stop-Process -Name siro -Force -ErrorAction SilentlyContinue

Write-Host "=== 1. Building Release Binary ===" -ForegroundColor Cyan
cargo build --release

if (!(Test-Path "target\dist")) {
    New-Item -ItemType Directory -Path "target\dist" | Out-Null
}

Copy-Item "target\release\siro.exe" "target\dist\siro.exe" -Force
Copy-Item "target\release\siro.exe" "target\dist\siro_portable.exe" -Force
Write-Host "Portable binaries created: target\dist\siro.exe & siro_portable.exe" -ForegroundColor Green

Write-Host "`n=== 2. Building Inno Setup Installer (Siro_Setup.exe) ===" -ForegroundColor Cyan
$isccPath = $null
$isccCmd = Get-Command iscc.exe -ErrorAction SilentlyContinue
if ($isccCmd) {
    $isccPath = $isccCmd.Source
} else {
    $searchPaths = @(
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\iscc.exe",
        "C:\Program Files (x86)\Inno Setup 6\iscc.exe",
        "C:\Program Files\Inno Setup 6\iscc.exe"
    )
    foreach ($p in $searchPaths) {
        if (Test-Path $p) {
            $isccPath = $p
            break
        }
    }
}

if ($isccPath -and (Test-Path $isccPath)) {
    & $isccPath "installer\siro_setup.iss"
    Write-Host "Inno Setup installer created: target\dist\Siro_Setup.exe" -ForegroundColor Green
} else {
    Write-Warning "Inno Setup compiler (iscc.exe) not found. Run: winget install JRSoftware.InnoSetup"
}

Write-Host "`n=== 3. Building WiX MSI Package (Siro.msi) ===" -ForegroundColor Cyan
$candlePath = $null
$lightPath = $null

$candleCmd = Get-Command candle.exe -ErrorAction SilentlyContinue
$lightCmd = Get-Command light.exe -ErrorAction SilentlyContinue

if ($candleCmd -and $lightCmd) {
    $candlePath = $candleCmd.Source
    $lightPath = $lightCmd.Source
} else {
    $wixDirs = @(
        "C:\Program Files (x86)\WiX Toolset v3.14\bin",
        "C:\Program Files (x86)\WiX Toolset v3.11\bin",
        "C:\Program Files\WiX Toolset v3.14\bin",
        "C:\Program Files\WiX Toolset v3.11\bin"
    )
    foreach ($d in $wixDirs) {
        if ((Test-Path "$d\candle.exe") -and (Test-Path "$d\light.exe")) {
            $candlePath = "$d\candle.exe"
            $lightPath = "$d\light.exe"
            break
        }
    }
}

if ($candlePath -and $lightPath) {
    & $candlePath "installer\siro.wxs" -out "target\dist\siro.wixobj"
    & $lightPath "target\dist\siro.wixobj" -out "target\dist\Siro.msi" -ext WixUIExtension
    Write-Host "WiX MSI package created: target\dist\Siro.msi" -ForegroundColor Green
} else {
    Write-Warning "WiX toolset (candle.exe / light.exe) not found. Run: winget install WiXToolset.WiXToolset"
}

if (!(Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist" | Out-Null
}
Copy-Item "target\dist\*" "dist\" -Include *.exe,*.msi -Force

Write-Host "`n=== All Deliverables Built Successfully in dist\ ===" -ForegroundColor Yellow
Get-ChildItem "dist" -Include *.exe,*.msi | Select-Object Name, Length, LastWriteTime
