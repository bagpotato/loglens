#!/usr/bin/env bash
# Emits a realistic mix of syslog events through logger(1) so LogLens has data to show.
# Usage: ./generate-logs.sh [--loop] [--interval SECONDS]
set -euo pipefail

LOOP=false
INTERVAL=10

usage() {
  echo "Usage: $0 [--loop] [--interval SECONDS]" >&2
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --loop) LOOP=true ;;
    --interval)
      [[ $# -ge 2 && "$2" =~ ^[0-9]+$ ]] || usage
      INTERVAL=$2
      shift
      ;;
    -h | --help) usage ;;
    *) usage ;;
  esac
  shift
done

command -v logger >/dev/null 2>&1 || {
  echo "logger not found (install util-linux or bsdutils)" >&2
  exit 1
}

ATTACKERS=(203.0.113.17 203.0.113.42 198.51.100.23 198.51.100.77 192.0.2.150)
USERS=(admin oracle test ubuntu postgres git guest)
PATHS=(/ /api/orders /api/users/42 /login /static/app.js /health /api/search?q=boots)
AGENTS=("Mozilla/5.0 (X11; Linux x86_64)" "curl/8.5.0" "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X)")

pick() {
  local -n arr=$1
  printf '%s' "${arr[RANDOM % ${#arr[@]}]}"
}

port() { printf '%d' $((40000 + RANDOM % 20000)); }
pid() { printf '%d' $((1000 + RANDOM % 30000)); }
emit() { logger -p "$1" -t "$2" -- "$3"; }

ssh_bruteforce() {
  local ip user i
  ip=$(pick ATTACKERS)
  for i in 1 2 3 4; do
    user=$(pick USERS)
    emit auth.info sshd "Invalid user $user from $ip port $(port)"
    emit auth.notice sshd "Failed password for invalid user $user from $ip port $(port) ssh2"
  done
  emit auth.notice sshd "Failed password for root from $ip port $(port) ssh2"
  emit auth.info sshd "Connection closed by authenticating user root $ip port $(port) [preauth]"
}

ssh_normal() {
  emit auth.info sshd "Accepted publickey for deploy from 10.0.0.5 port $(port) ssh2: ED25519 SHA256:q1w2e3r4t5y6u7i8o9p0"
  emit auth.info sshd "pam_unix(sshd:session): session opened for user deploy(uid=1000) by (uid=0)"
}

kernel_oom() {
  local p
  p=$(pid)
  emit user.warning kernel "java invoked oom-killer: gfp_mask=0x140cca(GFP_HIGHUSER_MOVABLE|__GFP_COMP), order=0, oom_score_adj=0"
  emit user.err kernel "oom-kill:constraint=CONSTRAINT_NONE,nodemask=(null),cpuset=/,mems_allowed=0,task=java,pid=$p,uid=1001"
  emit user.err kernel "Out of memory: Killed process $p (java) total-vm:8123456kB, anon-rss:4012345kB, file-rss:0kB"
}

kernel_io() {
  emit user.err kernel "blk_update_request: I/O error, dev sdb, sector $((RANDOM * 64)) op 0x0:(READ) flags 0x0 phys_seg 1 prio class 0"
  emit user.crit kernel "EXT4-fs (sdb1): Remounting filesystem read-only"
}

nginx_line() {
  local status=$1 ip method path bytes now
  ip="198.51.100.$((RANDOM % 254 + 1))"
  method=GET
  path=$(pick PATHS)
  bytes=$((200 + RANDOM % 5000))
  now=$(LC_ALL=C date '+%d/%b/%Y:%H:%M:%S %z')
  emit local7.info nginx "$ip - - [$now] \"$method $path HTTP/1.1\" $status $bytes \"-\" \"$(pick AGENTS)\""
}

nginx_traffic() {
  local i
  for i in 1 2 3 4 5 6; do nginx_line 200; done
  nginx_line 304
  nginx_line 404
  nginx_line 401
  nginx_line 429
  nginx_line 502
  nginx_line 504
  emit local7.err nginx "upstream timed out (110: Connection timed out) while reading response header from upstream, upstream: \"http://127.0.0.1:3000/api/orders\""
}

cron_jobs() {
  emit cron.info CRON "(root) CMD (/usr/local/bin/backup.sh >/dev/null 2>&1)"
  emit cron.info CRON "(www-data) CMD (php /var/www/app/artisan schedule:run)"
}

sudo_events() {
  emit authpriv.notice sudo "deploy : TTY=pts/0 ; PWD=/home/deploy ; USER=root ; COMMAND=/usr/bin/systemctl restart nginx"
  emit authpriv.notice sudo "pam_unix(sudo:auth): authentication failure; logname=bob uid=1001 euid=0 tty=/dev/pts/1 ruser=bob rhost=  user=bob"
}

systemd_events() {
  emit daemon.info systemd "Stopping nginx.service - A high performance web server and a reverse proxy server..."
  emit daemon.info systemd "Started nginx.service - A high performance web server and a reverse proxy server."
  emit daemon.warning systemd "worker.service: Main process exited, code=exited, status=1/FAILURE"
  emit daemon.info worker "queue consumer exited with code 1"
  emit daemon.info systemd "Started session-42.scope - Session 42 of User deploy."
}

app_noise() {
  emit user.info myapp "GET /health 200 2ms"
  emit user.info myapp "processed $((RANDOM % 500)) jobs in $((RANDOM % 900)) ms"
  emit user.warning myapp "database connection timeout, retrying in 5s"
  emit user.info myapp "config key 'cache.legacy_mode' is deprecated and will be removed in 3.0"
  emit user.notice postfix "connect from localhost[127.0.0.1]"
}

round() {
  app_noise
  ssh_normal
  cron_jobs
  nginx_traffic
  ssh_bruteforce
  systemd_events
  sudo_events
  if ((RANDOM % 3 == 0)) || [[ "$LOOP" == false ]]; then
    kernel_oom
  fi
  if ((RANDOM % 4 == 0)) || [[ "$LOOP" == false ]]; then
    kernel_io
  fi
}

round
echo "Demo events written to the system journal/syslog."
if [[ "$LOOP" == true ]]; then
  echo "Looping every ${INTERVAL}s, Ctrl+C to stop."
  while true; do
    sleep "$INTERVAL"
    round
  done
fi
