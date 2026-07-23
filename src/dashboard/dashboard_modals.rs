//! Dashboard modal HTML templates.

use maud::{Markup, html};

/// Renders the modal dialogs used in the dashboard.
pub fn render_modals() -> Markup {
    html! {
        div class="modal" id="serverModal" style="display:none" {
            div class="modal-content" {
                h2 id="modalTitle" { "Add server" }
                form id="serverForm" {
                    input type="hidden" id="serverType" value="";
                    input type="hidden" id="serverDirection" value="both";
                    input type="hidden" id="serverName" value="";

                    div class="form-group" {
                        label { "Server address" }
                        input type="text" id="serverUrl" placeholder="http://emby-or-jellyfin:8096" required {};
                        p class="form-hint" { "Full browser link or host:port; paths stripped." }
                        p class="form-hint" id="serverTypeHint" { "Works with Emby or Jellyfin." }
                    }
                    div class="form-group" {
                        label { "API key" }
                        input type="password" id="serverKey" placeholder="API key from Emby/Jellyfin" autocomplete="off" {};
                        p class="form-hint" { "Editing a saved server: leave blank to keep the stored key. Test and Live both use the real key, not the masked dots." }
                        p class="form-hint" id="serverLiveHint" style="display:none" {}
                    }
                    div class="form-group" {
                        label { "Sync direction" }
                        div class="btn-group" {
                            button type="button" class="btn-radio active" data-dir="both" onclick="pickDirection('both')" { "Both ways" }
                            button type="button" class="btn-radio" data-dir="send" onclick="pickDirection('send')" { "Send only" }
                            button type="button" class="btn-radio" data-dir="receive" onclick="pickDirection('receive')" { "Receive only" }
                        }
                    }
                    div class="modal-actions" {
                        button type="button" class="btn" onclick="testConnection()" { "Test connection" }
                        div class="right" {
                            button type="button" class="btn" onclick="closeModal('serverModal')" { "Cancel" }
                            button type="submit" class="btn btn-primary" { "Save" }
                        }
                    }
                }
            }
        }

        div class="modal" id="settingsModal" style="display:none" {
            div class="modal-content" style="max-width:520px" {
                h2 { "Settings" }
                div class="form-group" {
                    label { "Sync threshold (seconds)" }
                    input type="number" id="syncThreshold" min="1" max="60" value="5" {};
                    p class="form-hint" { "Ignore near-duplicate progress updates within this window." }
                }
                div class="form-group" {
                    label { "Live sync" }
                    p class="form-hint" style="margin-bottom:8px" { "While people watch — what to copy as events happen." }
                    label class="check-row" { input type="checkbox" id="syncLivePlayed" checked; " Played (mark watched)" }
                    label class="check-row" { input type="checkbox" id="syncLivePosition" checked; " Position (resume point)" }
                    label class="check-row" { input type="checkbox" id="syncLiveFavorites" checked; " Favorites (heart)" }
                }
                div class="form-group" {
                    label { "Force sync" }
                    p class="form-hint" style="margin-bottom:8px" { "Historical backfill when you press Force sync." }
                    label class="check-row" { input type="checkbox" id="syncForcePlayed" checked; " Played history" }
                    label class="check-row" { input type="checkbox" id="syncForcePosition" checked; " In-progress positions" }
                    label class="check-row" { input type="checkbox" id="syncForceFavorites" checked; " Favorites" }
                    p class="form-hint" { "Force only pushes when the target is missing that state. Use Preview force to count without writing." }
                }
                div class="form-group" {
                    label { "User allowlist (optional)" }
                    textarea id="cfgUserAllowlist" rows="3" placeholder="alice&#10;bob" {};
                    p class="form-hint" { "Empty = all users. One name per line; linked aliases included." }
                }
                div class="form-group" {
                    label { "Ignore users (optional)" }
                    textarea id="cfgUserIgnorelist" rows="3" placeholder="guest&#10;kids" {};
                    p class="form-hint" { "Never live- or force-sync these people. Or Mapped users → Actions → Ignore." }
                }
                div class="form-group" {
                    label { "Username mappings (advanced text)" }
                    textarea id="cfgUserMappings" rows="4" placeholder="alice, alice_jf&#10;bob, Robert" {};
                    p class="form-hint" { "Or use Link users for a visual picker. One group per line, comma-separated names." }
                }
                div class="modal-actions" {
                    div {}
                    div class="right" {
                        button type="button" class="btn" onclick="closeModal('settingsModal')" { "Cancel" }
                        button type="button" class="btn btn-primary" onclick="saveSettings()" { "Save settings" }
                    }
                }
            }
        }

        div class="modal" id="mapUsersModal" style="display:none" {
            div class="modal-content" style="max-width:560px" {
                h2 { "Link users" }
                p class="form-hint" style="margin-bottom:12px" {
                    "Pick the same person on each server. Names do not need to match — this mapping tells StateSync who is who."
                }
                div class="form-group" {
                    label id="mapServerALabel" { "User on server A" }
                    select id="mapUserA" {}
                }
                div class="form-group" {
                    label id="mapServerBLabel" { "User on server B" }
                    select id="mapUserB" {}
                }
                div class="modal-actions" style="margin-bottom:16px" {
                    div {}
                    div class="right" {
                        button type="button" class="btn btn-primary" onclick="addLinkedUserMapping()" { "Link these users" }
                    }
                }
                div class="form-group" {
                    label { "Current links" }
                    div id="mapLinksList" class="map-links" {}
                }
                div class="modal-actions" {
                    div {}
                    div class="right" {
                        button type="button" class="btn" onclick="closeModal('mapUsersModal')" { "Close" }
                    }
                }
            }
        }

        div class="modal" id="userActionsModal" style="display:none" {
            div class="modal-content" style="max-width:420px" {
                h2 { "User actions" }
                p class="form-hint" style="margin-bottom:12px" {
                    "Choose a person, then Force sync, Ignore, or Clear watched. Click a name in the table first to pre-select."
                }
                div class="form-group" {
                    label { "User" }
                    select id="userActionsSelect" onchange="refreshUserActionsIgnoreBtn()" {}
                }
                div class="modal-actions" style="margin-top:8px;flex-wrap:wrap;gap:8px" {
                    div {}
                    div class="right" style="display:flex;gap:8px;flex-wrap:wrap" {
                        button type="button" class="btn" id="userActionsForceBtn" onclick="userActionsForce()" { "Force sync" }
                        button type="button" class="btn" id="userActionsIgnoreBtn" onclick="userActionsToggleIgnore()" { "Ignore" }
                        button type="button" class="btn btn-danger" id="userActionsClearBtn" onclick="userActionsClearWatched()" { "Clear watched" }
                    }
                }
                div class="modal-actions" {
                    div {}
                    div class="right" {
                        button type="button" class="btn" onclick="closeModal('userActionsModal')" { "Close" }
                    }
                }
            }
        }
    }
}
