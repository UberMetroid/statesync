#!/bin/sh
# Entrypoint for the statesync container.
#
# Runs as root (Docker default for ENTRYPOINT).
#  1. Applies the user-configured UMASK
#  2. Chowns /config and /app to PUID:PGID (default 99:100, Unraid's
#     'nobody' user). Fails silently on read-only mounts; the daemon
#     falls back to /app/config.json in that case.
#  3. Execs the daemon as PUID:PGID via su-exec.
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

# Drop to the configured uid and exec.
exec su-exec "$PUID:$PGID" /usr/local/bin/statesync "$@"