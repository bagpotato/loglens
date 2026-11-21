#!/usr/bin/env bash
# Installs Vector and the LogLens pipeline on Debian/Ubuntu (apt) or Fedora (dnf).
# Usage: sudo bash install-agent.sh http://loglens.example.com:8080 <LOGLENS_PUSH_TOKEN>
set -euo pipefail

INSTALL_DIR=/etc/vector/loglens
ENV_FILE=/etc/vector/loglens.env
DROPIN_DIR=/etc/systemd/system/vector.service.d
DROPIN_FILE="$DROPIN_DIR/loglens.conf"
SETUP_URL=https://install.datadoghq.com/vector/setup.sh
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SRC_DIR="$SCRIPT_DIR/vector"
TMP_DIR=""
STAGE_DIR=""

log() { printf '[loglens-agent] %s\n' "$*"; }
warn() { printf '[loglens-agent] WARNING: %s\n' "$*" >&2; }
die() {
  printf '[loglens-agent] ERROR: %s\n' "$*" >&2
  exit 1
}
cleanup() {
  if [[ -n "$TMP_DIR" ]]; then
    rm -rf "$TMP_DIR"
  fi
  if [[ -n "$STAGE_DIR" && -d "$STAGE_DIR" ]]; then
    rm -rf "$STAGE_DIR"
  fi
}
trap cleanup EXIT

usage() {
  echo "Usage: sudo $0 <loglens-url> <push-token>   (example: http://192.168.1.10:8080 \$LOGLENS_PUSH_TOKEN)" >&2
  exit 2
}

detect_pm() {
  if [[ -f /etc/debian_version ]] && command -v apt-get >/dev/null 2>&1; then
    echo apt
  elif command -v dnf >/dev/null 2>&1; then
    echo dnf
  else
    die "unsupported distribution: need Debian/Ubuntu (apt) or Fedora (dnf)"
  fi
}

pkg_install() {
  local pm=$1
  shift
  case "$pm" in
    apt) DEBIAN_FRONTEND=noninteractive apt-get install -y -q "$@" ;;
    dnf) dnf install -y -q "$@" ;;
  esac
}

install_vector() {
  local pm=$1
  if [[ "$pm" == apt ]]; then
    # A Vector source left by an interrupted run may fail here; ensure_apt_legacy_key repairs it.
    apt-get update -q || warn "apt-get update reported errors; continuing"
  fi
  # Vector's setup script needs awk and gpg; minimal images (Fedora WSL/containers) lack them.
  case "$pm" in
    apt) pkg_install "$pm" curl ca-certificates gnupg gawk ;;
    dnf) pkg_install "$pm" curl ca-certificates gnupg2 gawk ;;
  esac

  TMP_DIR=$(mktemp -d)
  curl -fsSL --proto '=https' --proto-redir '=https' "$SETUP_URL" -o "$TMP_DIR/vector-setup.sh"
  if [[ -f /etc/apt/sources.list.d/vector.list || -f /etc/yum.repos.d/vector.repo ]]; then
    log "Vector repository already configured; refreshing its signing keys"
    bash "$TMP_DIR/vector-setup.sh" --migrate || warn "could not refresh the Vector repository keys"
  else
    log "Configuring the official Vector package repository"
    bash "$TMP_DIR/vector-setup.sh" || warn "Vector repository setup reported an error; checking signing keys"
  fi
  case "$pm" in
    apt) ensure_apt_legacy_key ;;
    dnf) ensure_dnf_legacy_key ;;
  esac

  log "Installing Vector"
  pkg_install "$pm" vector
  command -v vector >/dev/null 2>&1 || die "vector binary not found after installation"
  log "Installed $(vector --version)"
}

# During Vector's 2026 key rotation the repo is still signed with Datadog's 2023 APT key, which
# the new setup script no longer imports. Add it, pinned by full fingerprint, only when apt asks.
ensure_apt_legacy_key() {
  local keyring=/usr/share/keyrings/vector-archive-keyring.gpg
  local fpr=5F1E256061D813B125E156E8E6266D4AC0962C7D
  local out
  out=$(apt-get update -q 2>&1 || true)
  if [[ "$out" == *"NO_PUBKEY ${fpr: -16}"* ]]; then
    log "Adding Datadog's legacy APT signing key (${fpr: -16})"
    curl -fsSL --proto '=https' "https://keys.datadoghq.com/DATADOG_APT_KEY_${fpr: -8}.public" -o "$TMP_DIR/legacy.asc"
    gpg --show-keys --with-colons "$TMP_DIR/legacy.asc" 2>/dev/null | grep -q "^fpr:*:$fpr:" ||
      die "downloaded legacy key does not match fingerprint $fpr"
    gpg --dearmor <"$TMP_DIR/legacy.asc" >>"$keyring"
  fi
  apt-get update -q >/dev/null || die "apt-get update failed for the Vector repository"
}

# Same rotation for the yum repo: its metadata is still signed with Datadog's 2023 RPM key.
ensure_dnf_legacy_key() {
  local keyfile=/etc/pki/rpm-gpg/RPM-GPG-KEY-vector
  local fpr=7408BFD56BC5BF0C361AAAE85D88EEA3B01082D3
  # dnf5 exits 0 when repo metadata fails its signature check, so test for the package itself.
  if [[ -z "$(dnf -q --refresh --repo=vector repoquery vector 2>/dev/null)" ]]; then
    log "Adding Datadog's legacy RPM signing key (${fpr: -16})"
    curl -fsSL --proto '=https' "https://keys.datadoghq.com/DATADOG_RPM_KEY_${fpr: -8}.public" -o "$TMP_DIR/legacy-rpm.asc"
    gpg --show-keys --with-colons "$TMP_DIR/legacy-rpm.asc" 2>/dev/null | grep -q "^fpr:*:$fpr:" ||
      die "downloaded legacy key does not match fingerprint $fpr"
    cat "$TMP_DIR/legacy-rpm.asc" >>"$keyfile"
    rpm --import "$TMP_DIR/legacy-rpm.asc"
    # The fingerprint is verified above, so accept dnf's repo-key import prompt for it.
    dnf -y -q --refresh --repo=vector makecache >/dev/null 2>&1 || true
  fi
  [[ -n "$(dnf -q --refresh --repo=vector repoquery vector 2>/dev/null)" ]] ||
    die "dnf could not load the Vector repository"
}

has_docker() {
  [[ -S /var/run/docker.sock ]]
}

copy_config() {
  log "Copying pipeline to $INSTALL_DIR"
  install -d -m 0755 /etc/vector
  STAGE_DIR=$(mktemp -d /etc/vector/.loglens.XXXXXX)
  local stage=$STAGE_DIR
  cp -R "$SRC_DIR"/. "$stage"/
  rm -rf "$stage/tests"
  if ! has_docker; then
    log "No Docker socket found; disabling the docker_logs source"
    rm -f "$stage/sources/docker.yaml"
    # Vector rejects an input pattern that matches no component, so drop it from normalize too.
    sed -i '/^[[:space:]]*-[[:space:]]*docker\*[[:space:]]*$/d' "$stage/transforms/normalize.yaml"
  fi
  find "$stage" -type d -exec chmod 0755 {} +
  find "$stage" -type f -exec chmod 0644 {} +
  rm -rf "$INSTALL_DIR"
  mv "$stage" "$INSTALL_DIR"
  STAGE_DIR=""
}

write_env() {
  log "Writing $ENV_FILE"
  local tmp
  tmp=$(mktemp "$ENV_FILE.XXXXXX")
  cat >"$tmp" <<EOF
LOGLENS_LOKI_URL="$LOKI_URL"
LOGLENS_PUSH_TOKEN="$PUSH_TOKEN"
VECTOR_DANGEROUSLY_ALLOW_ENV_VAR_INTERPOLATION=true
EOF
  chmod 0600 "$tmp"
  mv "$tmp" "$ENV_FILE"
}

write_dropin() {
  local bin
  bin=$(command -v vector)
  log "Writing systemd drop-in $DROPIN_FILE"
  install -d -m 0755 "$DROPIN_DIR"
  cat >"$DROPIN_FILE" <<EOF
[Service]
EnvironmentFile=$ENV_FILE
WorkingDirectory=$INSTALL_DIR
ExecStartPre=
ExecStartPre=$bin validate --skip-healthchecks --config-dir $INSTALL_DIR
ExecStart=
ExecStart=$bin --config-dir $INSTALL_DIR
ExecReload=
ExecReload=$bin validate --skip-healthchecks --config-dir $INSTALL_DIR
ExecReload=/bin/kill -HUP \$MAINPID
EOF
  chmod 0644 "$DROPIN_FILE"
}

add_groups() {
  id vector >/dev/null 2>&1 || die "the vector user does not exist; the package install looks incomplete"
  local group
  for group in systemd-journal adm docker; do
    if getent group "$group" >/dev/null; then
      usermod -aG "$group" vector
      log "vector user added to group $group"
    fi
  done
}

validate_config() {
  log "Validating configuration"
  (
    set -a
    # shellcheck disable=SC1090
    . "$ENV_FILE"
    set +a
    cd "$INSTALL_DIR"
    vector validate --skip-healthchecks --config-dir "$INSTALL_DIR"
  ) || die "vector validate failed"
}

start_service() {
  log "Enabling and restarting vector.service"
  systemctl daemon-reload
  systemctl enable vector >/dev/null
  systemctl restart vector
  sleep 3
  if ! systemctl is-active --quiet vector; then
    journalctl -u vector -n 40 --no-pager >&2 || true
    die "vector.service is not running"
  fi
}

check_loki() {
  if curl -fsS --max-time 5 "$LOKI_URL/healthz" >/dev/null 2>&1; then
    log "LogLens at $LOKI_URL is reachable"
  else
    warn "LogLens at $LOKI_URL is not reachable yet; Vector keeps retrying in the background"
  fi
}

main() {
  [[ $# -eq 2 ]] || usage
  LOKI_URL=${1%/}
  PUSH_TOKEN=$2
  local url_re='^https?://[][A-Za-z0-9._:@%/-]+$'
  [[ "$LOKI_URL" =~ $url_re ]] || die "invalid LogLens URL: $1"
  [[ "$PUSH_TOKEN" =~ ^[A-Za-z0-9._~-]{24,}$ ]] || die "invalid push token (use LOGLENS_PUSH_TOKEN from the server .env)"
  [[ $EUID -eq 0 ]] || die "run as root (sudo $0 <loglens-url> <push-token>)"
  [[ -d "$SRC_DIR/sources" && -d "$SRC_DIR/vrl" ]] || die "pipeline not found in $SRC_DIR"
  command -v systemctl >/dev/null 2>&1 || die "systemd is required"

  local pm
  pm=$(detect_pm)
  install_vector "$pm"
  copy_config
  write_env
  write_dropin
  add_groups
  validate_config
  start_service
  check_loki
  log "Done. Logs from $(cat /proc/sys/kernel/hostname) now flow to $LOKI_URL"
}

main "$@"
