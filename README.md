<p align="center">
  <a href="https://github.com/studio2201/statesync">
    <img src="assets/header.png" alt="StateSync banner" width="100%" height="450" style="max-height:450px;width:100%;height:auto;object-fit:contain;object-position:center;">
  </a>
</p>

# <img src="assets/icon.png" width="32" height="32" valign="middle"> StateSync

Watched, resume, and favorites synced across Emby and Jellyfin (and same-type pairs). No media files are moved. Dashboard needs no login on the LAN.

## Install

```bash
docker run -d --name statesync -p 4601:4601 -v statesync-config:/config ghcr.io/studio2201/statesync:latest
```

No environment variables required. Open [http://localhost:4601](http://localhost:4601).

Image: `ghcr.io/studio2201/statesync` tags `latest`, `0.28.x`, `v0.28.x` (current **v0.28.79**).

## One perfect example

1. Run the install command above.
2. **Add server** — paste Emby or Jellyfin URL and API key, Save. Type is auto-detected. Browser paths such as `/web/index.html#!/…` are reduced to host and port.
3. Add the other server the same way.
4. If usernames differ, **Link users**.
5. Play something on one server — live sync updates the other.
6. Optional history: **Preview force**, then **Force sync**. For one person: open **Mapped users → Actions**.

## Mapped users and Actions

The Mapped users table shows one column per server (names stay aligned).

| Control | Where | What it does |
|---------|--------|----------------|
| **Link users** | Header | Map the same person across servers |
| **Actions** | Header | Pick a person, then Force / Ignore / Clear watched |
| Click a name | Table | Pre-selects that person for Actions |

**Actions modal**

- **Force sync** — historical backfill for that person only (played, resume, favorites).
- **Ignore / Un-ignore** — skip live and mesh force for them (and linked aliases). Shown as `name · ignored` in the grid.
- **Clear watched** — mark all played items unwatched on every server for that person. Dedicated high-regret action; not force.

You can also maintain ignore lists and allowlists under Settings.

## What it syncs

| | Live | Force |
|--|------|--------|
| Played | yes | yes (skips if already equal) |
| Position / resume | yes | yes |
| Favorites | yes | yes |

**Not synced:** ratings, playlists, libraries, media files.

**Force behavior**

- Mesh among servers using send / receive / both directions.
- **Skip if equal** so re-runs stay fast when libraries already match.
- **Preview force** (dry run) counts changes without writing.
- Global force buttons cover everyone who is not ignored; per-user force is under **Actions**.

**Now playing** shows current sessions with **Primary** posters (library cover art, not screenshots), cached so thumbs do not flicker.

## Deploy targets

| Target | How |
|--------|-----|
| Docker | One-liner above |
| Unraid | Import `unraid/unraid-template.xml`; appdata `/mnt/user/appdata/statesync`; port **4601**; shell **sh** (BusyBox ash). If Emby/Jellyfin use **br0**, put StateSync on br0 too so it can reach them. |
| Compose | `container/docker-compose.yml` (port + volume only) |
| Binary | GitHub Release `statesync-linux-x86_64-*.tar.gz` (static musl) |

Release assets also include a versioned Unraid XML and compose file.

## Runtime defaults

| | Default |
|--|---------|
| Bind | `0.0.0.0:4601` |
| Config | `/config/config.json` (created on first save) |
| Auth | off (optional `STATESYNC_WEB_AUTH=bearer:…` if you need it) |
| Base image | Alpine Linux with BusyBox ash |
| User | PUID **99**, PGID **100** when unset (Unraid-friendly) |

### Useful optional env

| Variable | Default | Notes |
|----------|---------|--------|
| `STATESYNC_FORCE_RATE` | `5` | Force items per second (1–50). Raise to `15`–`25` if media servers can take it. |
| `STATESYNC_BIND` | `0.0.0.0:4601` | Listen address |
| `STATESYNC_ACCEPT_INVALID_CERTS` | off | Only for self-signed HTTPS you trust |
| `RUST_LOG` | `info` | Logging |

More detail: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Brand rules (icon vs header): [graphics/BRAND.md](graphics/BRAND.md). Agent build rules: [AGENT.md](AGENT.md).

## CLI

```bash
statesync --validate
statesync --sync-force --dry-run
statesync --sync-force
statesync --tui
```

## Links

- Issues: https://github.com/studio2201/statesync/issues
- Packages: https://github.com/studio2201/statesync/pkgs/container/statesync
- Releases: https://github.com/studio2201/statesync/releases

## License

Apache License 2.0. See [LICENSE](LICENSE).
