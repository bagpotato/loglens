# agent

Purpose: Vector pipeline that ships host logs to Loki with LogLens labels, plus its installer.
- `vector/sources`: `journald` (sshd, kernel, systemd, cron, sudo, auth, podman), `docker` (docker_logs), `files` (nginx, apache, auth.log; tails from end).
- `vector/transforms` + `vector/vrl`: `normalize` sets host, source, container, unit, message; `severity` maps journald PRIORITY, then regex rules win (`severity_source` = `rule` | `default`).
- `vector/sinks/loki.yaml`: labels host, source, severity, container (docker/podman only); line = JSON `{message, severity_source, unit}`.
- `vector/tests`: one `vector test` case per file (insert at `normalize`, assert on `severity`).
- `install-agent.sh <loglens-url> <push-token>`: apt/dnf install from the official Vector repo, copies `vector/` (minus tests) to `/etc/vector/loglens`,
  writes `/etc/vector/loglens.env`, adds a systemd drop-in (`--config-dir`, `WorkingDirectory`), joins systemd-journal/adm/docker, validates, restarts. Idempotent.

Env vars: `LOGLENS_LOKI_URL` (Loki base URL), `VECTOR_DANGEROUSLY_ALLOW_ENV_VAR_INTERPOLATION=true` (needed since Vector 0.57 for `${...}`).
VRL `file:` paths are relative to the working directory, so run Vector from inside `vector/`.
Run tests: `cd agent/vector && LOGLENS_LOKI_URL=http://localhost:8080 LOGLENS_PUSH_TOKEN=test-token-0123456789abcdef vector test --dangerously-allow-env-var-interpolation --config-dir .`
Validate: `cd agent/vector && LOGLENS_LOKI_URL=http://localhost:8080 LOGLENS_PUSH_TOKEN=test-token-0123456789abcdef vector validate --skip-healthchecks --dangerously-allow-env-var-interpolation --config-dir .`
Without a Docker socket the installer drops `sources/docker.yaml` (the `docker*` input then matches nothing, a warning only).
Dependencies: Vector (latest stable from the official repo; config written against 0.62), systemd, journald. Never use message, PID or IP as a label.
