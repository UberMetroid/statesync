#!/bin/sh
# Entrypoint for the statesync container.
#
# Runs as root (Docker default for ENTRYPOINT).
#  1. Applies the user-configured UMASK
#  2. Chowns /config and /app to PUID:PGID (default 99:100, Unraid's
#     'nobody' user). Fails silently on read-only mounts; the daemon
#     falls back to /app/config.json in that case.
#  3. Ensures STATESYNC_WEB_AUTH is set for non-loopback binds:
#     - if the operator set it, use it
#     - else load /config/.web_auth if present
#     - else generate bearer:<hex>, persist, and print once
#  4. Execs the daemon as PUID:PGID via su-exec.
#
# About PUID/PGID/UMASK:
#   These are the standard Unraid community-app variables (used by
#   binhex-syncthing, glances, ollama, etc.). On Unraid, PUID=99 is
#   the 'nobody' user; setting PUID=99 makes the appdata dir show
#   as 'nobody' in the Unraid file manager instead of as a bare
#   numeric uid.
#
# su-exec is ~10KB of C that does the standard "drop privs and exec"
# pattern without the overhead of bash function spawning.

set -e

PUID=${PUID:-99}
PGID=${PGID:-100}
UMASK=${UMASK:-022}

# Apply umask so any files created later (logs, atomic-write temp
# files) have the right default permissions.
umask "$UMASK"

# Chown the persistent volume and workdir to the configured uid.
# If /config is read-only (host bind-mount with restrictive perms),
# chown fails and the daemon falls back to /app/config.json.
chown -R "$PUID:$PGID" /config 2>/dev/null || true
chown -R "$PUID:$PGID" /app 2>/dev/null || true

# Make sure the daemon binary is executable by the configured uid.
chmod +x /usr/local/bin/statesync

# --- Dashboard auth for non-loopback (Unraid host network) ---
# The daemon refuses 0.0.0.0 binds without STATESYNC_WEB_AUTH.
# Empty / "none" means "not set" — auto-provision from /config/.web_auth
# or generate a new token so first boot never hard-fails.
auth_is_empty() {
  v=$(printf '%s' "${STATESYNC_WEB_AUTH:-}" | tr -d '[:space:]')
  [ -z "$v" ] || [ "$(printf '%s' "$v" | tr '[:upper:]' '[:lower:]')" = "none" ]
}

if auth_is_empty; then
  if [ -f /config/.web_auth ]; then
    # shellcheck disable=SC2155
    export STATESYNC_WEB_AUTH=$(tr -d '\r\n' </config/.web_auth)
    echo "statesync: loaded STATESYNC_WEB_AUTH from /config/.web_auth"
  else
    # Prefer openssl if present; fall back to /dev/urandom hex.
    if command -v openssl >/dev/null 2>&1; then
      token=$(openssl rand -hex 32)
    else
      token=$(od -An -tx1 -N32 /dev/urandom 2>/dev/null | tr -d ' \n')
    fi
    if [ -z "$token" ]; then
      echo "statesync: FATAL: could not generate auth token and STATESYNC_WEB_AUTH is unset." >&2
      echo "statesync: Set STATESYNC_WEB_AUTH=bearer:<token> in the container template." >&2
      exit 1
    fi
    export STATESYNC_WEB_AUTH="bearer:${token}"
    # Best-effort persist (config volume may be missing on first create).
    if mkdir -p /config 2>/dev/null; then
      printf '%s\n' "$STATESYNC_WEB_AUTH" >/config/.web_auth
      chown "$PUID:$PGID" /config/.web_auth 2>/dev/null || true
      chmod 600 /config/.web_auth 2>/dev/null || true
      echo "statesync: generated STATESYNC_WEB_AUTH and saved to /config/.web_auth"
    else
      echo "statesync: generated STATESYNC_WEB_AUTH (could not write /config/.web_auth)"
    fi
    echo "statesync: ============================================================"
    echo "statesync:  Dashboard bearer token (save this):"
    echo "statesync:  ${token}"
    echo "statesync:  (full value: ${STATESYNC_WEB_AUTH})"
    echo "statesync:  Paste the token into the web UI auth prompt."
    echo "statesync: ============================================================"
  fi
fi

# Drop to the configured uid and exec.
# Export auth so the daemon process inherits it after su-exec.
export STATESYNC_WEB_AUTH
exec su-exec "$PUID:$PGID" /usr/local/bin/statesync "$@"
