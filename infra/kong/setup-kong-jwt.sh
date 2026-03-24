#!/usr/bin/env bash
# Keycloak の JWKS エンドポイントから RSA 公開鍵を取得し、
# kong.dev.yaml のプレースホルダーを実際の公開鍵に差し替えるセットアップスクリプト。
#
# 前提条件:
#   - jq がインストール済みであること
#   - Keycloak コンテナが起動済みであること（http://localhost:8180 でアクセス可能）
#   - openssl がインストール済みであること
#
# 使用方法:
#   ./infra/kong/setup-kong-jwt.sh
#
# べき等実行:
#   公開鍵が既に設定されている場合（プレースホルダーが存在しない場合）はスキップする。
#
# C-2 対応: kong.dev.yaml の JWT RSA 鍵プレースホルダーを Keycloak から取得した実鍵に自動置換する

set -euo pipefail

# 変数定義
KONG_DEV_YAML="$(dirname "$0")/kong.dev.yaml"
KEYCLOAK_HOST="${KEYCLOAK_HOST:-http://localhost:8180}"
REALM="${KEYCLOAK_REALM:-k1s0}"
JWKS_URL="${KEYCLOAK_HOST}/realms/${REALM}/protocol/openid-connect/certs"
PLACEHOLDER="....placeholder_replace_with_actual_keycloak_public_key...."

# 依存コマンドの確認
for cmd in jq curl; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "[ERROR] 必要なコマンドが見つかりません: ${cmd}" >&2
    echo "  インストール方法: sudo apt-get install ${cmd} または brew install ${cmd}" >&2
    exit 1
  fi
done

# プレースホルダーが存在しない場合は既に設定済みとしてスキップ
if ! grep -q "$PLACEHOLDER" "$KONG_DEV_YAML"; then
  echo "[INFO] kong.dev.yaml には既に RSA 公開鍵が設定されています。スキップします。"
  exit 0
fi

echo "[INFO] Keycloak JWKS エンドポイントから公開鍵を取得します: ${JWKS_URL}"

# Keycloak の起動を最大60秒待機する
MAX_RETRY=12
RETRY_INTERVAL=5
for i in $(seq 1 $MAX_RETRY); do
  if curl -sf "${KEYCLOAK_HOST}/health/ready" >/dev/null 2>&1; then
    echo "[INFO] Keycloak が起動しています。"
    break
  fi
  if [ "$i" -eq "$MAX_RETRY" ]; then
    echo "[ERROR] Keycloak が ${MAX_RETRY} 回の試行後も起動しませんでした。" >&2
    echo "  先に 'docker compose --profile infra up -d keycloak' を実行してください。" >&2
    exit 1
  fi
  echo "[INFO] Keycloak 起動待機中... (${i}/${MAX_RETRY})"
  sleep $RETRY_INTERVAL
done

# JWKS エンドポイントから RSA 署名用公開鍵（use: sig, kty: RSA）を取得する
echo "[INFO] JWKS から RSA 署名鍵を取得します..."
JWKS_JSON="$(curl -sf "$JWKS_URL")"

# kty: RSA かつ use: sig のキーを選択する
RSA_KEY="$(echo "$JWKS_JSON" | jq -r '[.keys[] | select(.kty == "RSA" and .use == "sig")] | first')"
if [ -z "$RSA_KEY" ] || [ "$RSA_KEY" = "null" ]; then
  echo "[ERROR] RSA 署名用キーが JWKS から取得できませんでした。" >&2
  echo "  JWKS レスポンス: $JWKS_JSON" >&2
  exit 1
fi

# JWK の n（modulus）と e（exponent）を取得する
N="$(echo "$RSA_KEY" | jq -r '.n')"
E="$(echo "$RSA_KEY" | jq -r '.e')"

if [ -z "$N" ] || [ "$N" = "null" ] || [ -z "$E" ] || [ "$E" = "null" ]; then
  echo "[ERROR] JWK から n または e パラメータを取得できませんでした。" >&2
  exit 1
fi

# JWK の n, e から PEM 形式の公開鍵を生成する（Python を使用）
PEM_KEY="$(python3 -c "
import base64, struct, sys

def decode_b64url(s):
    # Base64url デコード（パディング補完付き）
    padding = 4 - len(s) % 4
    s += '=' * (padding % 4)
    return base64.urlsafe_b64decode(s)

def int_to_bytes(n):
    # 整数をビッグエンディアンバイト列に変換する
    length = (n.bit_length() + 7) // 8
    return n.to_bytes(length, 'big')

n_bytes = decode_b64url('$N')
e_bytes = decode_b64url('$E')

n_int = int.from_bytes(n_bytes, 'big')
e_int = int.from_bytes(e_bytes, 'big')

# DER エンコード: RSAPublicKey (PKCS#1) を SubjectPublicKeyInfo (PKCS#8) に変換する
def encode_length(length):
    if length < 128:
        return bytes([length])
    elif length < 256:
        return bytes([0x81, length])
    else:
        return bytes([0x82, (length >> 8) & 0xFF, length & 0xFF])

def encode_integer(value):
    b = int_to_bytes(value)
    if b[0] & 0x80:
        b = b'\x00' + b
    return b'\x02' + encode_length(len(b)) + b

def encode_sequence(*elements):
    content = b''.join(elements)
    return b'\x30' + encode_length(len(content)) + content

# RSAPublicKey ::= SEQUENCE { modulus INTEGER, publicExponent INTEGER }
rsa_pub_key = encode_sequence(encode_integer(n_int), encode_integer(e_int))

# SubjectPublicKeyInfo ::= SEQUENCE { algorithm AlgorithmIdentifier, subjectPublicKey BIT STRING }
# OID for rsaEncryption: 1.2.840.113549.1.1.1
rsa_oid = b'\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x01\x05\x00'
bit_string = b'\x03' + encode_length(len(rsa_pub_key) + 1) + b'\x00' + rsa_pub_key
spki = encode_sequence(rsa_oid, bit_string)

pem = base64.encodebytes(spki).decode('ascii')
print('-----BEGIN PUBLIC KEY-----')
print(pem.strip())
print('-----END PUBLIC KEY-----')
" 2>&1)"

if [ $? -ne 0 ] || [ -z "$PEM_KEY" ]; then
  echo "[ERROR] PEM 形式の公開鍵生成に失敗しました。" >&2
  echo "  python3 が利用可能か確認してください。" >&2
  exit 1
fi

echo "[INFO] RSA 公開鍵を取得しました。kong.dev.yaml を更新します..."

# プレースホルダー行を実際の公開鍵コンテンツに置換する
# sed を使って PLACEHOLDER を含む行を削除し、その位置に公開鍵を挿入する
TEMP_FILE="$(mktemp)"

# 公開鍵の各行にインデント（10スペース）を付けてファイルに書き出す
PEM_INDENTED="$(echo "$PEM_KEY" | sed 's/^/          /')"

# PLACEHOLDER を含む行を公開鍵内容で置換する（インデント保持）
awk -v replacement="$PEM_INDENTED" '
/\.\.\.\.placeholder_replace_with_actual_keycloak_public_key\.\.\.\./ {
    print replacement
    next
}
{ print }
' "$KONG_DEV_YAML" > "$TEMP_FILE"

mv "$TEMP_FILE" "$KONG_DEV_YAML"

echo "[SUCCESS] kong.dev.yaml の RSA 公開鍵を更新しました。"
echo "  Kong を再起動して設定を反映してください:"
echo "    docker compose --profile infra restart kong"
