param(
    [string]$Version = "",
    [string]$OutputRoot = "dist",
    [switch]$SkipArchive
)

$ErrorActionPreference = "Stop"

function Resolve-Go {
    $go = Get-Command go -ErrorAction SilentlyContinue
    if ($go) {
        return $go.Source
    }

    $candidates = @(
        "C:\Program Files\Go\bin\go.exe",
        "C:\Program Files (x86)\Go\bin\go.exe",
        "$env:LOCALAPPDATA\Programs\Go\bin\go.exe"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate) {
            return $candidate
        }
    }

    throw "Go executable was not found. Install Go or add go.exe to PATH."
}

function Resolve-Version {
    param([string]$RequestedVersion)

    if ($RequestedVersion.Trim() -ne "") {
        return $RequestedVersion.Trim()
    }

    $git = Get-Command git -ErrorAction SilentlyContinue
    if ($git) {
        $description = & $git.Source describe --tags --always --dirty 2>$null
        if ($LASTEXITCODE -eq 0 -and $description.Trim() -ne "") {
            return $description.Trim()
        }
    }

    return "0.1.0-dev"
}

function New-RuntimeConfig {
    param([string]$TargetPath)

    $config = @'
{
  "app": {
    "display_name": "배움마루",
    "english_name": "Baeum-Maru",
    "mode": "portable"
  },
  "server": {
    "host": "127.0.0.1",
    "port": 18080
  },
  "database": {
    "path": "./data/center.db"
  },
  "backup": {
    "path": "./backups",
    "keep_days": 30
  },
  "export": {
    "path": "./exports"
  },
  "logging": {
    "path": "./logs/app.log",
    "level": "info"
  },
  "ui": {
    "open_browser_on_start": true
  }
}
'@
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($TargetPath, $config + [Environment]::NewLine, $utf8NoBom)
}

function New-GettingStarted {
    param(
        [string]$TargetPath,
        [string]$PackageVersion
    )

    $readme = @"
배움마루 Windows 포터블 패키지
버전: $PackageVersion

1. baeum-maru.exe를 실행합니다.
2. 브라우저가 자동으로 열리면 배움마루를 사용합니다.
3. 브라우저가 열리지 않으면 http://127.0.0.1:18080 으로 접속합니다.
4. 실행 중인 창을 닫거나 Ctrl+C를 누르면 서버가 종료됩니다.

폴더 안내
- data: SQLite 데이터베이스가 저장됩니다.
- backups: 백업 파일과 복원 예약 파일이 저장됩니다.
- exports: 엑셀 내보내기 파일이 저장됩니다.
- imports: 추후 가져오기 파일 작업용 폴더입니다.
- logs: 실행 로그가 저장됩니다.

주의
- data, backups, exports 폴더에는 개인정보가 포함될 수 있습니다.
- 복원 예약은 앱을 재시작할 때 적용됩니다.
"@
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($TargetPath, $readme + [Environment]::NewLine, $utf8NoBom)
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $repoRoot

$go = Resolve-Go
$packageVersion = Resolve-Version -RequestedVersion $Version
$safeVersion = $packageVersion -replace '[^\w\.-]', '-'
$packageName = "BaeumMaru_Portable_$safeVersion"
$packageDir = Join-Path $repoRoot (Join-Path $OutputRoot $packageName)
$archivePath = Join-Path $repoRoot (Join-Path $OutputRoot "$packageName.zip")

if (Test-Path -LiteralPath $packageDir) {
    Remove-Item -LiteralPath $packageDir -Recurse -Force
}
if ((Test-Path -LiteralPath $archivePath) -and -not $SkipArchive) {
    Remove-Item -LiteralPath $archivePath -Force
}

New-Item -ItemType Directory -Force -Path $packageDir | Out-Null
foreach ($dir in @("data", "backups", "exports", "imports", "logs")) {
    New-Item -ItemType Directory -Force -Path (Join-Path $packageDir $dir) | Out-Null
}

$ldflags = "-s -w -X github.com/Homeria/baeum-maru/internal/app.Version=$packageVersion"
& $go build -trimpath -ldflags $ldflags -o (Join-Path $packageDir "baeum-maru.exe") ./cmd/launcher
if ($LASTEXITCODE -ne 0) {
    throw "go build failed"
}

New-RuntimeConfig -TargetPath (Join-Path $packageDir "config.json")
New-GettingStarted -TargetPath (Join-Path $packageDir "README_FIRST_RUN.txt") -PackageVersion $packageVersion

if (-not $SkipArchive) {
    Compress-Archive -Path (Join-Path $packageDir "*") -DestinationPath $archivePath -Force
}

Write-Host "Package directory: $packageDir"
if (-not $SkipArchive) {
    Write-Host "Archive: $archivePath"
}
