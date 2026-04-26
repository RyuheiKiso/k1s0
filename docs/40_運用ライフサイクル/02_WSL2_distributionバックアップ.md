# 02. WSL2 distribution バックアップ Runbook

本 Runbook は Windows 11 + WSL2 (Ubuntu 24.04 LTS) ホストで k1s0 を運用する開発者向けに、`wsl --export` / `wsl --import` を使った distribution の**退避と復旧**手順を定める。設計書 [`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md`](../05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md) §7 で Phase 2 予定とされていた項目を実体化する。

「単一端末ですべて開発する」運用方針では、SSD 障害・WSL2 distribution の破損・誤操作で repo 含む環境が一括消失するリスクが SPOF（単一障害点）として顕在化する。GitHub への push が一次バックアップ、`wsl --export` による distribution まるごとの export が二次バックアップとなる。両者は補完関係で、push は repo 内のコミット済みコード、`--export` は未コミットのワークツリー・docker image / volume・ローカル secret・cargo target を含む全ホスト状態を救う。

## 1. 通常運用（incident でない場合）

### 1.1 月次バックアップ（手運用）

PowerShell（Windows host 側）で次のコマンドを実行する。実行に**先立って `wsl --shutdown` で distribution を一旦停止**するのが安全（実行中 distribution を export すると、稼働中の docker daemon の状態が中途半端な瞬間で固まる可能性がある）。

```powershell
# 1. distribution を停止
wsl --shutdown

# 2. 一覧を確認（distro 名と Version=2 を確認）
wsl --list --verbose

# 3. export（圧縮 tar 出力、5〜30GB 程度）
$BackupDir = "D:\wsl-backup"
$Date = Get-Date -Format "yyyyMMdd"
New-Item -ItemType Directory -Force -Path $BackupDir | Out-Null
wsl --export Ubuntu-24.04 "$BackupDir\Ubuntu-24.04-$Date.tar"

# 4. 検証（サイズが 5GB 以上、tar が壊れていない）
Get-ChildItem "$BackupDir\Ubuntu-24.04-$Date.tar" | Format-List Length, LastWriteTime
tar -tf "$BackupDir\Ubuntu-24.04-$Date.tar" | Select-Object -First 5

# 5. 古いバックアップを 3 世代に絞る
Get-ChildItem "$BackupDir\Ubuntu-24.04-*.tar" |
    Sort-Object LastWriteTime -Descending |
    Select-Object -Skip 3 |
    Remove-Item
```

`$BackupDir` は外付け SSD・NAS・OneDrive 同期フォルダなど、本端末の SSD とは異なる物理デバイスに置くこと。同 SSD 内に置くと、SSD 障害時に backup ごと失う。

### 1.2 復元（新端末・OS 再インストール後）

新端末で WSL2 を有効化した直後、Windows ストアから Ubuntu-24.04 を install せず、export した tar から直接 import する。

```powershell
# 1. WSL2 が有効か確認（無効なら有効化）
wsl --status

# 2. import 先を作成
$ImportDir = "C:\WSL\Ubuntu-24.04"
New-Item -ItemType Directory -Force -Path $ImportDir | Out-Null

# 3. import（5〜10 分）
wsl --import Ubuntu-24.04 $ImportDir "D:\wsl-backup\Ubuntu-24.04-20260424.tar" --version 2

# 4. 既定ユーザーを設定（vscode 等、export 時のユーザー名）
ubuntu2404 config --default-user vscode

# 5. 起動して動作確認
wsl -d Ubuntu-24.04 -- bash -c "whoami && cd ~ && ls && docker ps 2>&1 | head -3"
```

import 直後は docker daemon が停止状態で起動する。`sudo systemctl start docker` で起動するか、再ログインで systemd 経由で自動起動する。

### 1.3 export ファイルの差分確認

世代間で何が増えているかを把握しておくと、容量肥大の原因（cargo target / docker image 累積）を発見できる。

```powershell
$Old = "D:\wsl-backup\Ubuntu-24.04-20260301.tar"
$New = "D:\wsl-backup\Ubuntu-24.04-20260401.tar"
"Old: $((Get-Item $Old).Length / 1GB) GB"
"New: $((Get-Item $New).Length / 1GB) GB"
```

3GB 以上増えていれば WSL2 内で `du -sh /var/lib/docker /home/<user>/.cache /home/<user>/.cargo` を見て占有元を特定する。Phase 3 で容量制限の自動アラートを `40_運用ライフサイクル/` に追加する想定。

## 2. トラブルシュート（5 段構成）

### 2.1 `wsl --export` が `Access is denied` で失敗

#### 検出

- PowerShell で `wsl --export` 実行時に `Access is denied` または `0x8007005`
- 出力先ディレクトリの作成自体は成功している

#### 初動

```powershell
# 出力先の権限と空き容量を確認
Get-Acl "D:\wsl-backup" | Format-List Owner, AccessToString
Get-PSDrive D | Format-List Used, Free
# distribution が稼働中の場合は止める
wsl --shutdown
```

#### 復旧

```powershell
# 別ドライブに出力先を移す
$BackupDir = "$env:USERPROFILE\Documents\wsl-backup"
New-Item -ItemType Directory -Force -Path $BackupDir | Out-Null
wsl --export Ubuntu-24.04 "$BackupDir\Ubuntu-24.04-fallback.tar"
```

#### 根本原因調査

- 出力先が **Windows Defender Controlled Folder Access** の保護対象
- 出力先が ReFS / FAT32 で 4GB ファイル制限に抵触（NTFS なら問題なし）
- ウイルススキャナが tar 書き込みを部分的にブロック

#### 事後処理

- Defender の許可リストに `wsl.exe` を追加するか、保護対象外のフォルダを backup 先として固定
- 月次の cron 化を Phase 3 で検討（Windows タスクスケジューラ + PowerShell スクリプト）

### 2.2 `wsl --import` 後に docker daemon が起動しない

#### 検出

- import 直後の WSL2 で `docker ps` が `Cannot connect to the Docker daemon`
- `sudo systemctl status docker` が `Loaded: not-found`

#### 初動

```bash
# WSL2 内で systemd が動いているか
ps -p 1 -o comm=
# /etc/wsl.conf の boot 設定
cat /etc/wsl.conf
```

#### 復旧

`/etc/wsl.conf` に systemd 設定を入れて再起動。

```bash
sudo tee /etc/wsl.conf > /dev/null <<'EOF'
[boot]
systemd=true

[user]
default=vscode
EOF
```

PowerShell から `wsl --shutdown` 後、再起動。

```bash
# 復帰後
sudo systemctl enable --now docker
docker ps
```

#### 根本原因調査

- export 時点で systemd 未対応の WSL カーネル（< 0.67.6）だった可能性。`wsl --version` で 1.0+ を確認
- import 先の WSL2 が cgroups v1 で起動している場合、docker daemon が起動しても containerd が動かない（kind が失敗する）

#### 事後処理

- Phase 3 で `/etc/wsl.conf` の生成スクリプトを `tools/setup/wsl-bootstrap.sh` に置き、import 直後に手動で実行する手順を本 Runbook §1.2 に追記

### 2.3 export ファイルが想定より小さい（< 2GB）

#### 検出

- `Get-Item *.tar` で `Length` が想定（5〜10GB）より大幅に小さい
- `tar -tf` でリスト件数が極端に少ない

#### 初動

```powershell
# tar の整合性チェック
tar -tvf "D:\wsl-backup\Ubuntu-24.04-20260424.tar" | Measure-Object | Select-Object Count
# サイズ別 top 10
tar -tvf "D:\wsl-backup\Ubuntu-24.04-20260424.tar" |
    Sort-Object { -[int64]($_.Split()[2]) } |
    Select-Object -First 10
```

#### 復旧

```powershell
# 古いバックアップが正常なら、ロールバックして再 export
wsl --shutdown
wsl --export Ubuntu-24.04 "D:\wsl-backup\Ubuntu-24.04-retry.tar"
```

#### 根本原因調査

- distribution が `wsl --shutdown` できておらず、`--export` 中にプロセスが I/O で書き込んでいた
- WSL2 distribution 自体が破損している（このケースは `wsl -d Ubuntu-24.04 -- ls /home` でハングする）

#### 事後処理

- 月次バックアップの前に **必ず** `wsl --shutdown` を実行する手順を §1.1 に明記済み。逸脱が多発するなら PowerShell ラッパースクリプト化

## 3. 関連

- ホスト環境設計: [`docs/05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md`](../05_実装/50_開発者体験設計/05_ローカル環境基盤/01_WindowsWSL2環境構成.md) §7
- ADR: [ADR-DEV-002](../02_構想設計/adr/ADR-DEV-002-windows-wsl2-docker-runtime.md)
- IMP-DEV-ENV-060〜065（Windows + WSL2 + ネイティブ docker-ce）
- 関連 Runbook: [`01_ローカル本番再現スタック.md`](01_ローカル本番再現スタック.md)
