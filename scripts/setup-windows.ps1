# scripts/setup-windows.ps1
# Windows 開発環境セットアップスクリプト
# PowerShell で実行: Set-ExecutionPolicy -Scope CurrentUser RemoteSigned の後に .\scripts\setup-windows.ps1
# 前提条件チェック、Git設定、WSL2/devcontainer への誘導を行う

#Requires -Version 5.1

$ErrorActionPreference = "Stop"

# 出力ヘルパー関数
function Write-Header {
    param([string]$message)
    Write-Host ""
    Write-Host "=== $message ===" -ForegroundColor Cyan
}

function Write-OK {
    param([string]$message)
    Write-Host "  [OK]  $message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$message)
    Write-Host "  [!]   $message" -ForegroundColor Yellow
}

function Write-Fail {
    param([string]$message)
    Write-Host "  [NG]  $message" -ForegroundColor Red
}

function Write-Step {
    param([string]$message)
    Write-Host "  -->   $message" -ForegroundColor White
}

# ------------------------------------------------------------------
Write-Host ""
Write-Host "k1s0 Windows 開発環境セットアップ" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# ------------------------------------------------------------------
Write-Header "1. 前提条件チェック"

$allOk = $true

# Git for Windows チェック
if (Get-Command "git" -ErrorAction SilentlyContinue) {
    $gitVersion = git --version
    Write-OK "Git: $gitVersion"
} else {
    Write-Fail "Git が見つかりません"
    Write-Step "https://git-scm.com/download/win からインストールしてください"
    $allOk = $false
}

# Docker Desktop / Docker Engine チェック
if (Get-Command "docker" -ErrorAction SilentlyContinue) {
    try {
        $dockerVersion = docker --version
        Write-OK "Docker: $dockerVersion"
    } catch {
        Write-Warn "Docker はインストール済みですが起動していません（後で確認してください）"
    }
} else {
    Write-Fail "Docker が見つかりません"
    Write-Step "https://www.docker.com/products/docker-desktop/ からインストールしてください"
    Write-Step "WSL2 バックエンドを有効にしてください（Settings > General > Use WSL 2 based engine）"
    $allOk = $false
}

# VS Code チェック
$vsCodePaths = @(
    "$env:LOCALAPPDATA\Programs\Microsoft VS Code\Code.exe",
    "$env:ProgramFiles\Microsoft VS Code\Code.exe"
)
$vsCodeFound = $false
foreach ($path in $vsCodePaths) {
    if (Test-Path $path) {
        $vsCodeFound = $true
        break
    }
}
if ($vsCodeFound -or (Get-Command "code" -ErrorAction SilentlyContinue)) {
    Write-OK "VS Code: インストール済み"
} else {
    Write-Warn "VS Code が見つかりません（devcontainer を使う場合は必要）"
    Write-Step "https://code.visualstudio.com/ からインストールしてください"
}

# WSL2 チェック
try {
    $wslStatus = wsl --status 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-OK "WSL2: 利用可能"
    } else {
        Write-Warn "WSL2 がインストールされていません"
        Write-Step "管理者 PowerShell で: wsl --install -d Ubuntu-24.04"
    }
} catch {
    Write-Warn "WSL2 の確認に失敗しました（wsl コマンドが見つかりません）"
}

# ------------------------------------------------------------------
Write-Header "2. Git 設定"

# 改行コード設定（LF に統一）
git config --global core.autocrlf input
Write-OK "core.autocrlf = input (CRLF → LF 変換を無効化)"

# Windows パス長制限の回避
git config --global core.longpaths true
Write-OK "core.longpaths = true (260文字制限を回避)"

# ------------------------------------------------------------------
Write-Header "3. 開発環境セットアップの選択肢"

Write-Host ""
Write-Host "  k1s0 の開発には以下の3つの方法があります:" -ForegroundColor White
Write-Host ""
Write-Host "  【推奨】A: devcontainer（最速・全機能対応）" -ForegroundColor Green
Write-Host "    1. Docker Desktop を起動"
Write-Host "    2. VS Code に Dev Containers 拡張をインストール:"
Write-Host "       code --install-extension ms-vscode-remote.remote-containers"
Write-Host "    3. VS Code でリポジトリを開き F1 →"
Write-Host "       'Dev Containers: Reopen in Container' を選択"
Write-Host ""
Write-Host "  B: WSL2 ネイティブ（Rust/Go/TS/Dart 全対応）" -ForegroundColor Yellow
Write-Host "    WSL2 Ubuntu 内でリポジトリをクローンし:"
Write-Host "    bash scripts/setup-wsl.sh"
Write-Host ""
Write-Host "  C: Windows ネイティブ（CLI・TS・Dart 開発限定）" -ForegroundColor Gray
Write-Host "    Rust をインストール後: just cli-build / cli-test / cli-lint / cli-fmt"
Write-Host "    ※ サーバー開発・統合テスト・Docker Compose は WSL2/devcontainer が必要"
Write-Host ""

# ------------------------------------------------------------------
Write-Header "4. VS Code Dev Containers 拡張のインストール"

if (Get-Command "code" -ErrorAction SilentlyContinue) {
    Write-Step "Dev Containers 拡張をインストール中..."
    try {
        code --install-extension ms-vscode-remote.remote-containers
        Write-OK "Dev Containers 拡張をインストールしました"
    } catch {
        Write-Warn "拡張のインストールに失敗しました。VS Code から手動でインストールしてください"
    }
} else {
    Write-Warn "code コマンドが見つからないため拡張の自動インストールをスキップします"
    Write-Step "VS Code を起動し Extensions から 'Dev Containers' を検索してインストールしてください"
}

# ------------------------------------------------------------------
Write-Header "セットアップ完了"

if ($allOk) {
    Write-Host ""
    Write-Host "  前提条件がすべて満たされています。" -ForegroundColor Green
    Write-Host "  VS Code でリポジトリを開き、devcontainer を起動してください。" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "  上記の [NG] 項目を解消してから再度このスクリプトを実行してください。" -ForegroundColor Yellow
}

Write-Host ""
