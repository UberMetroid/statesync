//! Dashboard UI rendering module.
//!
//! Provides the Maud HTML templates, styling, and JavaScript logic for the web dashboard.

use maud::{DOCTYPE, Markup, html};

pub mod styles;
pub mod scripts;
pub mod scripts_actions;
pub mod scripts_modals;

/// Concatenates the embedded Rust JavaScript string slices into a single string for HTML insertion.
pub fn render_full_js() -> String {
    format!("{}{}{}", scripts::JS_CORE, scripts_actions::JS_ACTIONS, scripts_modals::JS_MODALS)
}

/// Renders the complete HTML dashboard markup using Maud templates.
pub fn render_dashboard() -> Markup {
    let full_js = render_full_js();
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta name="theme-color" content="#03060f";
                title { "StateSync" }
                link rel="manifest" href="/manifest.json";
                link rel="apple-touch-icon" href="/icon.svg";
                link rel="shortcut icon" href="/favicon.jpg" type="image/jpeg";
                link href="https://fonts.googleapis.com/css2?family=Share+Tech+Mono&display=swap" rel="stylesheet";
                style {
                    (maud::PreEscaped(styles::CSS))
                }
            }
            body {
                div class="container" {
                    h1 {
                        div style="display: flex; align-items: center; gap: 12px;" {
                            img src="/favicon.jpg" style="width: 36px; height: 36px; border-radius: 4px; border: 1px solid var(--border);" alt="";
                            span { "StateSync" }
                        }
                        div style="display: flex; gap: 10px; align-items: center;" {
                            button class="btn" id="refreshUsersBtn" onclick="refreshUsers()" { "[ REFRESH USERS ]" }
                            button class="btn btn-accent" id="forceSyncBtn" onclick="forceSync()" { "[ FORCE SYNC ]" }
                            button class="btn btn-accent" onclick="openSettingsModal()" { "[ SETTINGS ]" }
                            button class="btn" onclick="openServerModal(-1)" { "[ + ADD MEDIA SERVER ]" }
                        }
                    }
                    div id="lastFullSyncBanner" style="margin-bottom:20px;padding:10px 14px;border:1px solid rgba(255,255,255,0.1);background:rgba(0,0,0,0.2);font-size:12px;color:var(--text);display:flex;justify-space:between;align-items:center" {}
                    div id="forceSyncLive" style="margin-bottom:20px;padding:12px 14px;border:1px solid var(--border);background:rgba(0,240,255,0.06);font-size:12px;display:none" {
                        div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px" {
                            div { "FULL SYNC IN PROGRESS" }
                            div id="fsProgressText" style="color:var(--border)" {}
                        }
                        progress id="fsProgressBar" value="0" max="100" style="width:100%;height:8px;-webkit-appearance:none;appearance:none" {}
                        div id="fsCurrentUser" style="margin-top:6px;font-size:11px;color:var(--text);opacity:0.8" {}
                        div style="margin-top:8px;text-align:right" {
                            button class="btn btn-danger" id="fsCancelBtn" onclick="cancelForceSync()" { "CANCEL" }
                        }
                    }
                    div class="row-grid" {
                        div class="card" style="display:flex; flex-direction:column; height: 100%; box-sizing: border-box;" {
                            h2 { "[ MAPPED USERS ]" }
                            div id="syncedUsers" style="display: flex; flex-direction: column; gap: 8px; flex-grow: 1;" {}
                            div id="forceSyncStatus" style="margin-top:10px;font-size:11px;color:var(--text);opacity:0.7" {}
                        }
                        div style="display: flex; flex-direction: column; gap: 25px;" {
                            div class="card" {
                                h2 { "[ ACTIVE STREAMS ]" }
                                div id="activeSessions" style="display: flex; flex-direction: column; gap: 10px;" {
                                    div style="color: var(--accent)" { "NO ACTIVE STREAMS DETECTED" }
                                }
                            }
                            div class="card" {
                                h2 { "[ MEDIA SERVERS ]" }
                                div id="serverList" style="display: flex; flex-direction: column; gap: 8px;" {}
                            }
                        }
                    }
                    div class="card" style="margin-top: 20px;" {
                        div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:15px;" {
                            h2 style="margin-bottom:0;" { "[ TERMINAL LOG FEED ]" }
                            button class="btn" id="toggleLogsBtn" onclick="toggleLogs()" { "[ COLLAPSE ]" }
                        }
                        div class="log-feed" id="syncLogs" {}
                    }
                    div style="margin-top:20px;display:flex;justify-content:space-between;align-items:center;font-size:11px;color:var(--text);opacity:0.6" {
                        div id="versionFooter" {}
                        div style="display:flex;gap:12px;align-items:center" {
                            label for="themeSelector" style="font-size:11px" { "THEME:" }
                            select id="themeSelector" onchange="setTheme(this.value)" style="background:#000;border:1px solid var(--border-dim);color:var(--text);font-size:11px;padding:2px 6px" {
                                option value="cyberpunk" { "CYBERPUNK (CYAN)" }
                                option value="matrix" { "MATRIX (GREEN)" }
                                option value="amber" { "RETRO (AMBER)" }
                                option value="dracula" { "DRACULA (PURPLE)" }
                            }
                        }
                    }
                }
                div class="modal" id="serverModal" style="display:none" {
                    div class="modal-content" {
                        h2 id="modalTitle" { "[ ADD MEDIA SERVER ]" }
                        form id="serverForm" {
                            input type="hidden" id="serverType" value="jellyfin";
                            input type="hidden" id="serverDirection" value="both";
                            div class="form-group" {
                                label { "SERVER TYPE" }
                                div style="display:flex;gap:10px" {
                                    button type="button" class="btn-radio active" id="btnJellyfin" onclick="pickType('jellyfin')" { "JELLYFIN" }
                                    button type="button" class="btn-radio" id="btnEmby" onclick="pickType('emby')" { "EMBY" }
                                }
                            }
                            div class="form-group" {
                                label { "SYNC DIRECTION" }
                                div style="display:flex;gap:6px" {
                                    button type="button" class="btn-radio active" data-dir="both" onclick="pickDirection('both')" { "BIDIRECTIONAL" }
                                    button type="button" class="btn-radio" data-dir="send" onclick="pickDirection('send')" { "SEND ONLY" }
                                    button type="button" class="btn-radio" data-dir="receive" onclick="pickDirection('receive')" { "RECEIVE ONLY" }
                                }
                            }
                            div class="form-group" {
                                label { "SERVER ADDRESS (URL)" }
                                div style="display:flex;gap:8px" {
                                    input type="text" id="serverUrl" placeholder="http://192.168.1.10:8096" required style="flex-grow:1" {};
                                    button type="button" class="btn" id="autoNameBtn" onclick="autoFetchServerName()" title="Auto-detect server name" { "↻ AUTO" }
                                }
                            }
                            div class="form-group" {
                                label { "API KEY / TOKEN" }
                                input type="password" id="serverKey" required {};
                            }
                            div class="form-group" {
                                label { "SERVER NAME" }
                                input type="text" id="serverName" placeholder="Living Room Jellyfin" required {};
                            }
                            div style="display:flex;justify-content:space-between;margin-top:20px" {
                                button type="button" class="btn" onclick="testConnection()" { "[ PING LINK ]" }
                                div style="display:flex;gap:10px" {
                                    button type="button" class="btn" onclick="closeModal('serverModal')" { "[ CANCEL ]" }
                                    button type="submit" class="btn btn-accent" { "[ SAVE ]" }
                                }
                            }
                        }
                    }
                }
                div class="modal" id="settingsModal" style="display:none" {
                    div class="modal-content" style="width:520px" {
                        h2 { "[ SYSTEM SETTINGS ]" }
                        div class="form-group" {
                            label { "SYNC THRESHOLD (SECONDS)" }
                            input type="number" id="syncThreshold" min="1" max="60" value="5" {};
                            p style="font-size:10px;color:var(--text);margin-top:4px" { "Ignore duplicate progress events within N seconds." }
                        }
                        div class="form-group" {
                            label { "CUSTOM USERNAME MAPPINGS" }
                            textarea id="cfgUserMappings" rows="5" placeholder="alice, Alice, alice_jellyfin&#10;bob, Robert" {};
                            p style="font-size:10px;color:var(--text);margin-top:4px" { "One mapping group per line, comma separated usernames across servers." }
                        }
                        div style="display:flex;justify-content:flex-end;margin-top:20px;gap:10px" {
                            button type="button" class="btn" onclick="closeModal('settingsModal')" { "[ CANCEL ]" }
                            button type="button" class="btn btn-accent" onclick="saveSettings()" { "[ SAVE SETTINGS ]" }
                        }
                    }
                }
                div class="toast" id="toast" {}
                div class="modal" id="authModal" style="display:none" {
                    div class="modal-content" {
                        h2 { "[ AUTHENTICATION REQUIRED ]" }
                        p style="color: var(--text); font-size: 12px; margin-bottom: 12px;" {
                            "This dashboard is protected. Enter the bearer token configured on the server."
                        }
                        div class="form-group" {
                            label { "BEARER TOKEN" }
                            input type="password" id="authToken" autocomplete="off" {}
                        }
                        div style="display:flex;justify-content:flex-end;margin-top:20px;gap:12px" {
                            button class="btn btn-accent" id="authSubmitBtn" { "[ UNLOCK ]" }
                        }
                    }
                }
                script {
                    (maud::PreEscaped(full_js))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::render_dashboard;

    #[test]
    fn test_render_dashboard_contains_title() {
        let html_str = render_dashboard().into_string();
        assert!(html_str.contains("<title>StateSync</title>"));
    }

    #[test]
    fn test_render_dashboard_contains_headings() {
        let html_str = render_dashboard().into_string();
        assert!(html_str.contains("[ MAPPED USERS ]"));
        assert!(html_str.contains("[ ACTIVE STREAMS ]"));
        assert!(html_str.contains("[ MEDIA SERVERS ]"));
        assert!(html_str.contains("[ TERMINAL LOG FEED ]"));
    }
}
