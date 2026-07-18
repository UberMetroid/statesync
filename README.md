# Emby Sync Play Daemon

A lightweight, high-performance Rust daemon designed to synchronize playback across multiple Emby Media Server client sessions. Perfect for watch parties, home setups with multiple TVs, or synchronized multi-room playback.

## Features

- **Multi-Leader Sync**: Play, pause, seek, or change media on *any* of the configured TVs, and all other TVs will instantly synchronize.
- **Choose to Join (Sync Rooms)**: Instead of joining automatically, you can navigate to a custom library in Emby on your TV and select which room you want to join using your TV remote.
- **WebSocket Driven**: Subscribes to real-time session events on Emby, reducing network overhead and providing sub-second reaction times.
- **Smart Lag/Buffering Correction**: If a TV is buffering or lagging behind, it is commanded to seek forward to catch up *without* dragging the other TVs backward.
- **Automatic Cooldowns**: Prevents feedback loops where one TV's state update triggers commands that echo back.
- **Dynamic Reconnection**: Automatically reconnects to the Emby WebSocket in case of connection dropouts or server restarts.
- **Startup Sessions Inspector**: Queries and lists all active client sessions and their Device IDs at startup to simplify configuration.

## How it Works

The daemon runs an asynchronous event loop that:
1. Listens for session updates on Emby via WebSockets.
2. Intercepts when a TV client plays one of the dummy files in the "Sync Rooms" library (e.g. `Sync to Living Room`). It stops that dummy playback and instantly redirects your TV client to join the target room's movie at its current playback position.
3. Automatically generates the room trigger video files inside `./sync_rooms/` on startup.

---

## Configuration

The daemon is configured via a `config.json` file in its current working directory.

### Example `config.json`

```json
{
  "emby_url": "http://192.168.1.100:8096",
  "api_key": "YOUR_EMBY_API_KEY",
  "sync_devices": [
    { "id": "device_id_of_tv_1", "name": "Living Room" },
    { "id": "device_id_of_tv_2", "name": "Bedroom" },
    { "id": "device_id_of_tv_3", "name": "Kitchen" }
  ],
  "sync_threshold_seconds": 3,
  "cooldown_seconds": 5
}
```

### Configuration Fields

- `emby_url`: The URL of your Emby Media Server (e.g., `http://192.168.1.100:8096`).
- `api_key`: A valid Emby API key. Generate one from the Emby dashboard under **Settings** -> **API Keys**.
- `sync_devices`: A list of target devices to sync, specified as objects:
  - `id`: The unique `DeviceId` of the client.
  - `name`: A friendly name (e.g. `Living Room`). This name will be used to generate the trigger video file `Sync to {name}.mp4`.
- `sync_threshold_seconds`: The maximum difference (in seconds) allowed between client positions before a seek command is triggered. Default: `3`.
- `cooldown_seconds`: Cooldown duration (in seconds) applied to a device after commanding it, ignoring its transient status reports. Default: `5`.

---

## How to Set Up the "Sync Rooms" Library

1. **Find TV Device IDs**: Start the daemon with dummy Device IDs. At startup, the daemon will print all active client sessions and their details:
   ```text
   [INFO] Successfully connected to Emby server. Active sessions found:
   [INFO]   - Device: 'LG OLED', Client: 'Emby for LG Smart TV', User: 'Jeryd', DeviceId: 'a1b2c3d4-e5f6...'
   ```
2. **Configure your Devices**: Copy the `DeviceId`s and update the `sync_devices` array in `config.json` with their IDs and friendly names.
3. **Run the Daemon**: Run the daemon once. It will automatically create a folder named `sync_rooms` in your project folder, and generate a 5-second video file for each friendly room name (e.g., `Sync to Living Room.mp4`).
4. **Add the Library in Emby**: 
   - Go to your Emby Server Dashboard.
   - Go to **Library** -> **Add Media Library**.
   - Select **Content Type**: `Home videos & photos` (or `Mixed Content`).
   - Set **Display Name**: `Sync Rooms`.
   - Add the folder path pointing to your daemon's generated `sync_rooms` directory (e.g. `/home/jeryd/Projects/emby-syncplay/sync_rooms`).
   - Click **Ok** and let Emby scan the library.
5. **How to Sync**: On your TV, navigate to the `Sync Rooms` row on your home screen or sidebar. Highlight the poster card for the room you want to join (e.g. `Sync to Living Room`) and click **Play**. Within a second, the TV app will transition directly into the movie currently playing in the Living Room, perfectly synced!

---

## Running the Daemon

Set the log filter environment variable to see output, and run:

```bash
RUST_LOG=info cargo run
```

Or build a release version:

```bash
cargo build --release
./target/release/emby-syncplay
```
