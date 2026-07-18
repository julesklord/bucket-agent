# install.ps1 — Install bucket-agent from GitHub Releases
# Usage: irm https://raw.githubusercontent.com/julesklord/bucket-agent/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "julesklord/bucket-agent"
$BinaryName = "bucket.exe"
$InstallDir = Join-Path $env:USERPROFILE ".bucket\bin"

function Detect-Platform {
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
    return "windows-$arch"
}

function Download-Release {
    param(
        [string]$Platform,
        [string]$Version = ""
    )
    
    if ([string]::IsNullOrEmpty($Version)) {
        Write-Host "Fetching latest version..."
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $Version = $release.tag_name -replace '^v', ''
    }
    
    Write-Host "Downloading bucket-agent v$Version for $Platform..."
    
    $AssetName = "bucket-$Version-$Platform.exe"
    $DownloadUrl = "https://github.com/$Repo/releases/download/v$Version/$AssetName"
    
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    
    $destPath = Join-Path $InstallDir $BinaryName
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $destPath
    
    Write-Host "Installed bucket-agent v$Version to $destPath"
}

function Setup-Path {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "User")
        Write-Host "Added $InstallDir to PATH"
        Write-Host "Please restart your terminal or run: `$env:Path += `";$InstallDir`""
    }
}

function Main {
    $platform = Detect-Platform
    $version = if ($args.Count -gt 0) { $args[0] } else { "" }
    
    Download-Release -Platform $platform -Version $version
    Setup-Path
    
    Write-Host ""
    Write-Host "Installation complete! Run 'bucket' to start."
}

Main @args