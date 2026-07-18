mod config;
mod client;
mod state;
mod websocket;

use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context};
use tracing::info;
use tracing_subscriber;

use crate::config::Config;
use crate::client::MediaClient;
use crate::state::{AppState, init_server_cache};
use crate::websocket::{make_ws_url, handle_websocket_loop};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Multi-Server Playstate Sync Sidecar...");

    let config = Config::load()?;
    let mut clients = Vec::new();
    let mut caches = Vec::new();

    // Initialize all clients and cache metadata
    for s in &config.servers {
        info!("Connecting to server '{}' ({})", s.name, s.url);
        let client = Arc::new(MediaClient::new(s.url.clone(), s.api_key.clone(), s.is_emby));
        
        info!("Initializing metadata cache for '{}'...", s.name);
        let cache = init_server_cache(&s.name, &client).await
            .with_context(|| format!("Failed to initialize cache for server '{}'", s.name))?;
        
        info!("Cache loaded for '{}': {} users, {} matched media items.", s.name, cache.users.len(), cache.id_to_providers.len());
        
        clients.push(client);
        caches.push(cache);
    }

    let app_state = Arc::new(Mutex::new(AppState::new(caches)));

    // Spawn WebSocket tasks for each server
    for (i, s) in config.servers.iter().enumerate() {
        let ws_url = make_ws_url(&s.url, &s.api_key, s.is_emby);
        let state_clone = app_state.clone();
        let config_clone = config.clone();

        // Target clients list is all other clients
        let mut target_clients = Vec::new();
        for (j, client) in clients.iter().enumerate() {
            if j != i {
                target_clients.push((j, client.clone()));
            }
        }

        tokio::spawn(async move {
            handle_websocket_loop(
                i,
                &ws_url,
                target_clients,
                state_clone,
                config_clone,
            ).await;
        });
    }

    info!("Multi-Server Playstate Sync Sidecar started. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("Stopping Multi-Server Playstate Sync Sidecar.");
    Ok(())
}
