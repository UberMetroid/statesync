//! Style definitions for the StateSync web dashboard.

/// CSS stylesheet embedded into the HTML dashboard.
pub const CSS: &str = r#":root {
  --bg: #03060f;
  --card-bg: rgba(6, 12, 24, 0.85);
  --border: #00f0ff;
  --border-dim: rgba(0, 240, 255, 0.2);
  --text: #a0aec0;
  --text-bright: #ffffff;
  --accent: #ff0055;
  --green: #00ff66;
  --red: #ff2a2a;
}
body.theme-matrix {
  --bg: #020b05;
  --card-bg: rgba(4, 20, 10, 0.85);
  --border: #00ff66;
  --border-dim: rgba(0, 255, 102, 0.2);
  --text: #73b885;
  --accent: #00ff66;
}
body.theme-amber {
  --bg: #0b0702;
  --card-bg: rgba(20, 14, 4, 0.85);
  --border: #ffb000;
  --border-dim: rgba(255, 176, 0, 0.2);
  --text: #b89c73;
  --accent: #ffb000;
}
body.theme-dracula {
  --bg: #0d0a14;
  --card-bg: rgba(22, 16, 36, 0.85);
  --border: #bd93f9;
  --border-dim: rgba(189, 147, 249, 0.2);
  --text: #a899c7;
  --accent: #ff79c6;
}
* { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Share Tech Mono', monospace; }
body { background: var(--bg); color: var(--text); padding: 20px; }
.container { max-width: 1200px; margin: 0 auto; }
h1 { color: #fff; text-transform: uppercase; margin-bottom: 20px; font-size: 24px; border-bottom: 2px solid var(--border); padding-bottom: 10px; display: flex; justify-content: space-between; align-items: center; }
.card { background: var(--card-bg); border: 1px solid var(--border); padding: 20px; border-radius: 4px; box-shadow: 0 0 15px var(--border-dim); margin-bottom: 20px; }
h2 { color: var(--border); font-size: 16px; margin-bottom: 15px; text-transform: uppercase; }
.server-row { display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.05); margin-bottom: 8px; border-radius: 3px; }
.server-info { display: flex; gap: 10px; align-items: center; }
.badge { background: rgba(0,240,255,0.1); border: 1px solid var(--border); color: var(--border); font-size: 10px; padding: 2px 6px; border-radius: 2px; }
.btn { background: transparent; border: 1px solid var(--border); color: var(--border); padding: 6px 12px; cursor: pointer; font-size: 12px; transition: all 0.2s; }
.btn:hover { background: var(--border); color: #000; }
.btn-danger { border-color: var(--red); color: var(--red); }
.btn-danger:hover { background: var(--red); color: #fff; }
.btn-accent { border-color: var(--accent); color: var(--accent); }
.btn-accent:hover { background: var(--accent); color: #fff; }
.btn-radio { background: transparent; border: 1px solid var(--border-dim); color: var(--text); padding: 4px 10px; cursor: pointer; font-size: 11px; transition: all 0.2s; }
.btn-radio.active { background: var(--border); color: #000; font-weight: bold; border-color: var(--border); }
.btn-radio[data-dir].active { background: var(--accent); color: #fff; border-color: var(--accent); }
.user-cell { padding: 6px 10px; font-size: 12px; text-align: center; border: 1px solid rgba(255,255,255,0.05); background: rgba(0,0,0,0.2); }
.user-cell.filled { color: #fff; border-color: var(--border-dim); background: rgba(0,240,255,0.05); }
.user-cell.empty { color: var(--text); opacity: 0.4; }
.row-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
.log-feed { background: #000; border: 1px solid var(--border-dim); padding: 10px; font-size: 11px; height: 180px; overflow-y: auto; color: #a0aec0; }
.log-line { margin-bottom: 4px; word-break: break-all; }
.status-Connected { color: var(--green); }
.status-Error { color: var(--red); }
.status-Offline { color: var(--text); }
.toast { position: fixed; bottom: 20px; right: 20px; background: #000; border: 1px solid var(--border); color: var(--border); padding: 10px 20px; font-size: 12px; display: none; z-index: 100; }
.modal { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.8); display: flex; justify-content: center; align-items: center; z-index: 50; }
.modal-content { background: var(--bg); border: 1px solid var(--border); width: 420px; padding: 20px; border-radius: 4px; }
.form-group { margin-bottom: 12px; }
.form-group label { display: block; font-size: 11px; color: var(--text); margin-bottom: 4px; }
.form-group input, .form-group select, .form-group textarea { width: 100%; background: #000; border: 1px solid var(--border-dim); color: #fff; padding: 8px; font-size: 12px; }
@media (max-width: 768px) { .row-grid { grid-template-columns: 1fr; } h1 { flex-direction: column; gap: 10px; align-items: flex-start; } }
"#;
