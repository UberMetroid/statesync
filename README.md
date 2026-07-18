# statesync

A lightweight, high-performance Rust daemon designed to synchronize playback progress, watch states, and resume points bi-directionally between an Emby Media Server and a Jellyfin Media Server in real-time.

## Features

- **Bi-directional Real-Time Sync**: Syncs playback positions, play states, and paused/resumed statuses between Emby and Jellyfin instantly.
- **IMDb & TMDb Matching**: Uses global identifiers (IMDb ID and TMDb ID) from the metadata of your media files to link items. Works perfectly even if database IDs, filenames, or library structures differ between your servers.
- **LDAP-Friendly User Mapping**: Matches users across servers automatically by matching their usernames (case-insensitive). Perfect for setups synced via LDAP or Active Directory.
- **Intelligent Feedback Loop Prevention**: Caches and tracks the last synchronized positions per user/movie to prevent endless "ping-pong" update loops between servers.
- **Robust Connection Recovery**: Connects to the WebSockets of both servers concurrently and automatically reconnects in case of connection dropouts or server restarts.
- **Zero Server Modification**: Requires no plugins, DLLs, or restarts on either Emby or Jellyfin. Connects purely via standard REST APIs and WebSockets.

---

## Configuration

`statesync` can be configured using either **Environment Variables** or a **`config.json`** file.

### Option A: Environment Variables (Recommended for Containers)

Set the following environment variables when running the service:

- `STATESYNC_EMBY_URL`: The URL of your Emby Media Server.
- `STATESYNC_EMBY_API_KEY`: A valid Emby API key.
- `STATESYNC_JELLYFIN_URL`: The URL of your Jellyfin Media Server.
- `STATESYNC_JELLYFIN_API_KEY`: A valid Jellyfin API key.
- `STATESYNC_SYNC_THRESHOLD_SECONDS`: Optional. Sync threshold in seconds. Default: `5`.
- `RUST_LOG`: Logging verbosity level (`info`, `warn`, `error`, `debug`).

### Option B: `config.json` File

Create a file named `config.json` at either `/etc/statesync/config.json`, `/app/config.json`, or in the daemon's working directory:

```json
{
  "emby": {
    "url": "http://192.168.3.3:8096",
    "api_key": "YOUR_EMBY_API_KEY"
  },
  "jellyfin": {
    "url": "http://192.168.3.10:8096",
    "api_key": "YOUR_JELLYFIN_API_KEY"
  },
  "sync_threshold_seconds": 5
}
```

---

## Container Deployment

We package `statesync` as a lightweight container using **RedHat UBI-minimal (`ubi9/ubi-minimal`)** as the secure base runtime image.

### 1. Run with Docker Compose (Recommended)

1. Create a `docker-compose.yml` file:
   ```yaml
   version: '3.8'
   services:
     statesync:
       build: .
       container_name: statesync
       restart: unless-stopped
       environment:
         - STATESYNC_EMBY_URL=http://192.168.3.3:8096
         - STATESYNC_EMBY_API_KEY=YOUR_EMBY_API_KEY
         - STATESYNC_JELLYFIN_URL=http://192.168.3.10:8096
         - STATESYNC_JELLYFIN_API_KEY=YOUR_JELLYFIN_API_KEY
         - STATESYNC_SYNC_THRESHOLD_SECONDS=5
         - RUST_LOG=info
   ```
2. Build and start the container:
   ```bash
   docker compose up -d --build
   ```

### 2. Run with Docker Volume Mounts (Using `config.json`)

If you prefer using a configuration file instead of environment variables:

```bash
docker run -d \
  --name statesync \
  -v /path/to/config.json:/etc/statesync/config.json:ro \
  -e RUST_LOG=info \
  statesync:latest
```

---

## Local Development (Without Containers)

1. Install Cargo and Rust.
2. Build and run locally:
   ```bash
   RUST_LOG=info cargo run
   ```
