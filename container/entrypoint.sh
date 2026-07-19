#!/bin/sh
# Entrypoint for the statesync container.
#
# Runs as root (Docker default for ENTRYPOINT).
#  1. Best-effort chown of /config and /app to nobody:nogroup so the
#     daemon (which we drop to nobody below) can write to them. Fails
#     silently on read-only mounts; the daemon handles that gracefully.
#  2. Execs the daemon as nobody:nogroup via su-exec.
#
# About the user:
#   Alpine's /etc/passwd has:  nobody:x:65534:65534:nobody:/:/sbin/nologin
#   So inside the container, `ls -l` shows owner `nobody` (uid 65534).
#   If you see the bare numeric `65534` in some other view, that's
#   because that view is consulting the host's /etc/passwd (where the
#   same uid 65534 typically also maps to nobody). 65534 IS nobody;
#   it's the same user, just shown numerically.
#
# su-exec is ~10KB of C that does the standard "drop privs and exec"
# pattern without the overhead of bash function spawning.

set -e

# Chown the persistent volume and workdir to nobody. If /config is
# read-only (host bind-mount with restrictive perms), chown fails
# and the daemon falls back to /app/config.json (see config.rs).
chown -R nobody:nogroup /config 2>/dev/null || true
chown -R nobody:nogroup /app 2>/dev/null || true

# Make sure the daemon binary is executable by nobody.
chmod +x /usr/local/bin/statesync

# Drop to nobody and exec. su-exec ensures a clean transition with no
# bash subshell or signal-forwarding issues.
exec su-exec nobody:nogroup /usr/local/bin/statesync "$@"