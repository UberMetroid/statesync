//! Full dashboard HTML template.

use maud::{DOCTYPE, Markup, html};

/// Renders the complete HTML dashboard markup using Maud templates.
pub fn render_dashboard() -> Markup {
    let full_js = super::render_full_js();
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta name="theme-color" content="#0b0f14";
                title { "StateSync" }
                link rel="manifest" href="/manifest.json";
                link rel="icon" href="/favicon.jpg" type="image/jpeg";
                link rel="shortcut icon" href="/favicon.jpg" type="image/jpeg";
                link rel="apple-touch-icon" href="/favicon.jpg";
                style { (maud::PreEscaped(super::styles::css_full())) }
            }
            body {
                div class="container" {
                    div class="header" {
                        div class="brand" {
                            img src="/favicon.jpg" alt="StateSync" width="32" height="32";
                            span { "StateSync" }
                        }
                        div class="actions" {
                            button class="btn" id="refreshUsersBtn" onclick="refreshUsers()" { "Refresh users" }
                            button class="btn" id="previewForceBtn" onclick="forceSync(true)" { "Preview force" }
                            button class="btn btn-primary" id="forceSyncBtn" onclick="forceSync(false)" { "Force sync" }
                            button class="btn" onclick="openSettingsModal()" { "Settings" }
                            button class="btn btn-primary" onclick="openServerModal(-1)" { "Add server" }
                        }
                    }

                    div id="lastFullSyncBanner" class="banner" {}
                    div id="forceSyncLive" class="banner banner-live" style="display:none" {
                        div class="fs-live-top" {
                            div class="fs-live-top-text" {
                                strong id="fsStoryTitle" style="color:var(--bright)" { "Force sync running" }
                                span id="fsProgressText" style="color:var(--accent);font-size:12px" {}
                            }
                            div class="fs-live-actions" {
                                button class="btn fs-btn-details" id="fsStoryToggleBtn" type="button" onclick="toggleForceStory()" { "Details" }
                                button class="btn btn-danger" id="fsCancelBtn" type="button" onclick="cancelForceSync()" { "Cancel" }
                            }
                        }
                        progress id="fsProgressBar" value="0" max="100" class="fs-progress" {}
                        div id="fsCurrentUser" class="form-hint" style="margin-top:6px" {}
                        div id="fsStoryExpanded" class="fs-story-expanded" style="display:none" {
                            div id="fsStoryDetail" class="fs-fact-block" {}
                            div id="fsFailureList" class="fs-failure-list" style="display:none" {}
                        }
                    }

                            (super::dashboard_how::how_sync_card())
                    div class="row-grid" {
                        div class="card" {
                            div style="display:flex;justify-content:space-between;align-items:center;gap:10px;margin-bottom:12px;flex-wrap:wrap" {
                                h2 style="margin:0" { "Mapped users" }
                                div style="display:flex;gap:8px;flex-wrap:wrap" {
                                    button class="btn" onclick="openMapUsersModal()" { "Link users" }
                                    button class="btn" id="userActionsBtn" onclick="openUserActionsModal()" { "Actions" }
                                }
                            }
                            div id="syncedUsers" {}
                        }
                        div class="stack" {
                            div class="card" {
                                h2 { "Now playing" }
                                div id="activeSessions" {
                                    div class="empty" { "No one is playing anything right now." }
                                }
                            }
                            div class="card" {
                                h2 { "Media servers" }
                                div id="serverList" {}
                            }
                        }
                    }

                    div class="card" {
                        div style="display:flex;justify-content:space-between;align-items:center;gap:10px;margin-bottom:12px;flex-wrap:wrap" {
                            h2 style="margin:0" { "Activity log" }
                            div style="display:flex;gap:8px" {
                                button class="btn" id="copyLogsBtn" onclick="copyActivityLog()" { "Copy log" }
                                button class="btn" id="toggleLogsBtn" onclick="toggleLogs()" { "Collapse" }
                            }
                        }
                        div class="log-feed" id="syncLogs" {}
                    }

                    div class="footer" {
                        div id="versionFooter" {}
                    }
                }

                (super::dashboard_modals::render_modals())

                div class="toast" id="toast" {}

                script { (maud::PreEscaped(full_js)) }
            }
        }
    }
}
