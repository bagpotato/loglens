#!/usr/bin/env bash
# Creates .env from .env.example with freshly generated secrets.
# Usage: bash init-env.sh   (refuses to overwrite an existing .env)
set -euo pipefail

cd "$(dirname "$0")"
[[ -f .env ]] && { echo ".env already exists; edit it or remove it first" >&2; exit 1; }
[[ -f .env.example ]] || { echo ".env.example not found" >&2; exit 1; }

rand() { head -c "$1" /dev/urandom | od -An -vtx1 | tr -d ' \n'; }

admin_password=$(rand 12)
jwt_secret=$(rand 32)
push_token=$(rand 24)
ntfy_topic="loglens-$(rand 8)"

sed -e "s|^LOGLENS_ADMIN_PASSWORD=.*|LOGLENS_ADMIN_PASSWORD=$admin_password|" \
    -e "s|^LOGLENS_JWT_SECRET=.*|LOGLENS_JWT_SECRET=$jwt_secret|" \
    -e "s|^LOGLENS_PUSH_TOKEN=.*|LOGLENS_PUSH_TOKEN=$push_token|" \
    -e "s|^LOGLENS_NTFY_TOPIC=.*|LOGLENS_NTFY_TOPIC=$ntfy_topic|" \
    .env.example > .env
chmod 600 .env

cat <<EOF
.env created.
  Admin user:     admin
  Admin password: $admin_password
  Push token:     $push_token   (for agent/install-agent.sh)
  ntfy topic:     $ntfy_topic   (subscribe in the ntfy app)
These values are stored in .env only.
EOF
