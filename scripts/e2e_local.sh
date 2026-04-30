#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3306}"
MYSQL_USER="${MYSQL_USER:-evt}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-evt}"
MYSQL_DATABASE="${MYSQL_DATABASE:-evt_e2e}"
MYSQL_CLEAN_DATABASE="${MYSQL_CLEAN_DATABASE:-evt}"
HTTP_HOST="${HTTP_HOST:-127.0.0.1}"
HTTP_PORT="${HTTP_PORT:-18008}"
GRPC_HOST="${GRPC_HOST:-127.0.0.1}"
GRPC_PORT="${GRPC_PORT:-19020}"
TEST_USER="evt_e2e_$(date +%s)"
TEST_PASS="Passw0rd_123"
TMP_DIR="$(mktemp -d)"
SERVER_LOG="$TMP_DIR/server.log"
USE_ISOLATED_DB=1

cleanup() {
  if [[ -n "${TEST_USER_ID:-}" ]]; then
    mysql \
      --host="${MYSQL_HOST}" \
      --port="${MYSQL_PORT}" \
      --user="${MYSQL_USER}" \
      --password="${MYSQL_PASSWORD}" \
      --protocol=TCP \
      -D "${MYSQL_CLEAN_DATABASE}" \
      -e "DELETE FROM users WHERE id = ${TEST_USER_ID};" >/dev/null 2>&1 || true
  fi
  if [[ -n "${SERVER_PID:-}" ]] && kill -0 "${SERVER_PID}" 2>/dev/null; then
    kill "${SERVER_PID}" 2>/dev/null || true
    wait "${SERVER_PID}" 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"

echo "[e2e] building web"
yarn --cwd web build >/dev/null

echo "[e2e] preparing database"
if mysql \
  --host="${MYSQL_HOST}" \
  --port="${MYSQL_PORT}" \
  --user="${MYSQL_USER}" \
  --password="${MYSQL_PASSWORD}" \
  --protocol=TCP \
  -e "DROP DATABASE IF EXISTS \`${MYSQL_DATABASE}\`; CREATE DATABASE \`${MYSQL_DATABASE}\` CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;" >/dev/null 2>&1; then
  DATABASE_NAME="${MYSQL_DATABASE}"
  MYSQL_CLEAN_DATABASE="${MYSQL_DATABASE}"
else
  USE_ISOLATED_DB=0
  DATABASE_NAME="${MYSQL_CLEAN_DATABASE}"
fi

DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@${MYSQL_HOST}:${MYSQL_PORT}/${DATABASE_NAME}"

echo "[e2e] starting local rust backend"
EVT_RS__DATABASE__URL="${DATABASE_URL}" \
EVT_RS__SERVER__HTTP__HOST="${HTTP_HOST}" \
EVT_RS__SERVER__HTTP__PORT="${HTTP_PORT}" \
EVT_RS__SERVER__GRPC__HOST="${GRPC_HOST}" \
EVT_RS__SERVER__GRPC__PORT="${GRPC_PORT}" \
EVT_RS__STORAGE__LOCAL_DIR="${ROOT_DIR}/custom/data/attachments" \
cargo run --quiet -p evt-app >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

BASE_URL="http://${HTTP_HOST}:${HTTP_PORT}"

echo "[e2e] waiting for server"
for _ in $(seq 1 60); do
  if curl -sf "${BASE_URL}/healthz" >/dev/null; then
    break
  fi
  sleep 1
done

curl -sf "${BASE_URL}/healthz" >"$TMP_DIR/healthz.json"
curl -sf "${BASE_URL}/" >"$TMP_DIR/index.html"

ASSET_PATH="$(
  sed -n 's/.*src="\([^"]*\/assets\/[^"]*\.js\)".*/\1/p' "$TMP_DIR/index.html" | head -n 1
)"
test -n "$ASSET_PATH"
curl -sf "${BASE_URL}${ASSET_PATH}" >"$TMP_DIR/asset.js"
curl -sf "${BASE_URL}/v1/site/profile" >"$TMP_DIR/site_profile.json"

REGISTER_JSON="$(
  curl -sf -X POST "${BASE_URL}/v1/auth/register" \
    -H 'content-type: application/json' \
    -d "{\"username\":\"${TEST_USER}\",\"password\":\"${TEST_PASS}\"}"
)"

LOGIN_JSON="$(
  curl -sf -X POST "${BASE_URL}/v1/auth/login" \
    -H 'content-type: application/json' \
    -d "{\"username\":\"${TEST_USER}\",\"password\":\"${TEST_PASS}\"}"
)"

TOKEN="$(
  LOGIN_JSON="${LOGIN_JSON}" python - <<'PY'
import json, os
print(json.loads(os.environ["LOGIN_JSON"])["data"]["token"])
PY
)"

curl -sf "${BASE_URL}/v1/user/info" \
  -H "authorization: Bearer ${TOKEN}" >"$TMP_DIR/user_info.json"
curl -sf "${BASE_URL}/v1/users/me" \
  -H "authorization: Bearer ${TOKEN}" >"$TMP_DIR/current_user.json"

TEST_USER_ID="$(
  USER_INFO_JSON="$(cat "$TMP_DIR/user_info.json")" python - <<'PY'
import json, os
print(json.loads(os.environ["USER_INFO_JSON"])["data"]["id"])
PY
)"

POST_JSON="$(
  curl -sf -X POST "${BASE_URL}/v1/post" \
    -H 'content-type: application/json' \
    -H "authorization: Bearer ${TOKEN}" \
    -d '{"contents":[{"content":"evt e2e post","type":2,"sort":100}],"tags":[],"users":[],"attachment_price":0,"visibility":0}'
)"

POST_ID="$(
  POST_JSON="${POST_JSON}" python - <<'PY'
import json, os
print(json.loads(os.environ["POST_JSON"])["data"]["id"])
PY
)"

curl -sf "${BASE_URL}/v1/post?id=${POST_ID}" \
  -H "authorization: Bearer ${TOKEN}" >"$TMP_DIR/post.json"
curl -sf "${BASE_URL}/v1/posts?page=1&page_size=20" >"$TMP_DIR/posts.json"

COMMENT_JSON="$(
  curl -sf -X POST "${BASE_URL}/v1/post/comment" \
    -H 'content-type: application/json' \
    -H "authorization: Bearer ${TOKEN}" \
    -d "{\"post_id\":${POST_ID},\"contents\":[{\"content\":\"evt e2e comment\",\"type\":2,\"sort\":100}],\"users\":[]}"
)"

COMMENT_ID="$(
  COMMENT_JSON="${COMMENT_JSON}" python - <<'PY'
import json, os
print(json.loads(os.environ["COMMENT_JSON"])["data"]["id"])
PY
)"

curl -sf "${BASE_URL}/v1/post/comments?id=${POST_ID}&page=1&page_size=20" \
  -H "authorization: Bearer ${TOKEN}" >"$TMP_DIR/comments.json"

echo "[e2e] assertions"
TMP_DIR_ENV="$TMP_DIR" python - <<'PY'
import json
import os
from pathlib import Path

tmp = Path(os.environ["TMP_DIR_ENV"])

healthz = json.loads((tmp / "healthz.json").read_text())
assert healthz["code"] == 0
assert healthz["data"]["status"] == "ok"

index_html = (tmp / "index.html").read_text()
assert "<div id=\"app\"></div>" in index_html

asset_js = (tmp / "asset.js").read_text()
assert len(asset_js) > 100

site_profile = json.loads((tmp / "site_profile.json").read_text())
assert site_profile["code"] == 0
assert "allow_user_register" in site_profile["data"]

user_info = json.loads((tmp / "user_info.json").read_text())
assert user_info["code"] == 0
assert user_info["data"]["username"].startswith("evt_e2e_")

current_user = json.loads((tmp / "current_user.json").read_text())
assert current_user["code"] == 0
assert current_user["data"]["username"].startswith("evt_e2e_")

post = json.loads((tmp / "post.json").read_text())
assert post["code"] == 0
assert post["data"]["contents"][0]["content"] == "evt e2e post"

posts = json.loads((tmp / "posts.json").read_text())
assert posts["code"] == 0
assert isinstance(posts["data"]["list"], list)

comments = json.loads((tmp / "comments.json").read_text())
assert comments["code"] == 0
assert any(item["id"] > 0 for item in comments["data"]["list"])
PY

echo "[e2e] success"
printf 'user=%s\npost_id=%s\ncomment_id=%s\nasset=%s\n' \
  "$TEST_USER" "$POST_ID" "$COMMENT_ID" "$ASSET_PATH"
if [[ "$USE_ISOLATED_DB" -eq 1 ]]; then
  printf 'database_mode=isolated\n'
else
  printf 'database_mode=shared\n'
fi
