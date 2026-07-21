//! Core JavaScript client logic for the StateSync dashboard.

/// Core dashboard script string slice (Part 1: Initialization & Data Rendering).
pub const JS_CORE: &str = r#"if ('serviceWorker' in navigator) { navigator.serviceWorker.register('/sw.js').catch(() => {}); }
const $ = id => document.getElementById(id);
let currentConfig = { servers: [], sync_threshold_seconds: 5 }; let editIndex = -1;
const AUTH_TOKEN_KEY = 'statesync-auth-token';
function esc(s) { if (s == null) return ''; return String(s).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'})[c]); }
function getAuthHeaders() {
  const t = localStorage.getItem(AUTH_TOKEN_KEY);
  return t ? { 'Authorization': 'Bearer ' + t } : {};
}
async function authedFetch(url, opts) {
  opts = opts || {};
  opts.headers = Object.assign({}, opts.headers || {}, getAuthHeaders());
  const r = await fetch(url, opts);
  if (r.status === 401) { showAuthModal(); throw new Error('unauthorized'); }
  return r;
}
function showAuthModal() {
  const m = $('authModal'); if (m) m.style.display = 'flex';
}
function hideAuthModal() {
  const m = $('authModal'); if (m) m.style.display = 'none';
}
function submitAuth() {
  const t = $('authToken').value.trim();
  if (!t) return;
  localStorage.setItem(AUTH_TOKEN_KEY, t);
  hideAuthModal();
  loadDashboard();
}
function setTheme(n) { document.body.className = n === 'cyberpunk' ? '' : `theme-${n}`; localStorage.setItem('hud-theme', n); }
async function loadDashboard() {
  try {
    const [configRes, statusRes] = await Promise.all([
      authedFetch('/api/config'),
      authedFetch('/api/status')
    ]);
    currentConfig = await configRes.json(); const status = await statusRes.json();
    $('syncThreshold').value = currentConfig.sync_threshold_seconds;
    $('cfgUserMappings').value = (currentConfig.user_mappings || []).map(group => group.join(', ')).join('\n');
    const listDiv = $('serverList');
    if (currentConfig.servers.length === 0) {
      listDiv.textContent = '';
      const empty = document.createElement('div'); empty.style.color = 'var(--accent)'; empty.textContent = 'NO MEDIA SERVERS CONFIGURED';
      listDiv.appendChild(empty);
    } else {
      listDiv.textContent = '';
      currentConfig.servers.forEach((srv, idx) => {
        const sStatus = status.servers.find(s => s.name === srv.name) || { users_count: 0, media_count: 0, websocket_status: 'Offline' };
        const row = document.createElement('div'); row.className = 'server-row';
        const dirBadge = srv.sync_direction === 'send' ? ' [SEND ONLY]' : (srv.sync_direction === 'receive' ? ' [RCV ONLY]' : '');
        const urlText = (status.servers.find(s => s.name === srv.name) || {}).url || srv.url;

        const left = document.createElement('div'); left.className = 'server-info';
        const statusSpanEl = document.createElement('span'); statusSpanEl.className = 'status-' + sStatus.websocket_status;
        statusSpanEl.textContent = '[ ' + sStatus.websocket_status.toUpperCase() + ' ]';
        const leftInner = document.createElement('div');
        const nameEl = document.createElement('span'); nameEl.style.cssText = 'font-weight:600;color:#fff'; nameEl.textContent = srv.name;
        const badgeEl = document.createElement('span'); badgeEl.className = 'badge'; badgeEl.textContent = (srv.is_emby ? 'EMBY' : 'JELLYFIN') + dirBadge;
        const urlEl = document.createElement('div'); urlEl.style.cssText = 'font-size:11px;color:var(--text);margin-top:2px'; urlEl.textContent = urlText;
        leftInner.appendChild(nameEl); leftInner.appendChild(document.createTextNode(' ')); leftInner.appendChild(badgeEl); leftInner.appendChild(urlEl);
        left.appendChild(statusSpanEl); left.appendChild(leftInner);

        const right = document.createElement('div'); right.className = 'server-info';
        const metaSpan = document.createElement('span'); metaSpan.style.fontSize = '12px';
        if (sStatus.websocket_status === 'Scanning' || sStatus.websocket_status === 'Validating' || sStatus.websocket_status === 'Connecting') {
          metaSpan.textContent = sStatus.websocket_status.toUpperCase() + '...';
        } else {
          metaSpan.textContent = sStatus.users_count + ' USERS';
        }
        const editBtn = document.createElement('button'); editBtn.className = 'btn'; editBtn.textContent = '[ EDIT ]';
        editBtn.addEventListener('click', () => openServerModal(idx));
        const removeBtn = document.createElement('button'); removeBtn.className = 'btn btn-danger'; removeBtn.textContent = '[ REMOVE ]';
        removeBtn.addEventListener('click', () => deleteServer(idx));
        right.appendChild(metaSpan); right.appendChild(editBtn); right.appendChild(removeBtn);

        row.appendChild(left); row.appendChild(right);
        listDiv.appendChild(row);
      });
    }
    const activeDiv = $('activeSessions');
    if (status.active_sessions && status.active_sessions.length > 0) {
      activeDiv.textContent = '';
      status.active_sessions.forEach(sess => {
        const mins = Math.floor(sess.position / 60); const secs = Math.floor(sess.position % 60).toString().padStart(2, '0');
        const durationStr = mins + ':' + secs;
        const row = document.createElement('div'); row.className = 'server-row';
        if (sess.poster_url) { row.style.borderColor = 'var(--accent)'; row.style.padding = '6px 18px'; }
        const left = document.createElement('div'); left.className = 'server-info';
        if (sess.poster_url) {
          const img = document.createElement('img');
          img.src = sess.poster_url;
          img.alt = '';
          img.style.cssText = 'width:30px;height:45px;object-fit:cover;border:1px solid var(--accent);margin-right:12px;flex-shrink:0;';
          left.appendChild(img);
        }
        const meta = document.createElement('div');
        const itemEl = document.createElement('div'); itemEl.style.cssText = 'font-weight:600;color:#fff'; itemEl.textContent = sess.item;
        
        const userEl = document.createElement('div'); userEl.style.cssText = 'font-size:11px;color:var(--text)';
        if (sess.is_paused) {
          userEl.textContent = sess.user + ' paused on ' + sess.server + '. Position is locked at ' + durationStr + '.';
        } else {
          userEl.textContent = sess.user + ' is watching on ' + sess.server + '. Progress is actively syncing.';
        }
        
        meta.appendChild(itemEl); meta.appendChild(userEl);
        left.appendChild(meta);
        const right = document.createElement('div'); right.style.cssText = 'display:flex;align-items:center;gap:10px';
        const badge = document.createElement('span'); badge.className = 'badge'; badge.style.cssText = 'border-color:var(--accent);color:var(--accent)';
        badge.textContent = durationStr;
        right.appendChild(badge);
        if (sess.is_paused) {
          const p = document.createElement('span'); p.style.cssText = 'font-size:11px;color:var(--accent)'; p.textContent = '[ PAUSED ]';
          right.appendChild(p);
        }
        row.appendChild(left); row.appendChild(right);
        activeDiv.appendChild(row);
      });
    } else {
      activeDiv.textContent = '';
      const empty = document.createElement('div'); empty.style.color = 'var(--accent)'; empty.textContent = 'ALL QUIET. STATESYNC IS WAITING FOR SOMEONE TO PLAY A MOVIE OR SHOW.';
      activeDiv.appendChild(empty);
    }
    const usersDiv = $('syncedUsers');
    if (!status.servers || status.servers.length === 0) {
      usersDiv.textContent = '';
      const empty = document.createElement('div'); empty.style.color = 'var(--accent)'; empty.textContent = 'NO MEDIA SERVERS CONFIGURED';
      usersDiv.appendChild(empty);
    } else {
      usersDiv.textContent = '';
      const serverCount = status.servers.length;
      const headerRow = document.createElement('div');
      headerRow.style.cssText = 'display:grid;grid-template-columns:repeat(' + serverCount + ', 1fr);gap:6px;margin-bottom:6px';
      status.servers.forEach(srv => {
        const h = document.createElement('div');
        h.style.cssText = 'text-align:center;color:var(--border);font-weight:600;font-size:12px;padding-bottom:6px;border-bottom:1px solid rgba(0,240,255,0.3);text-transform:uppercase';
        h.textContent = srv.name;
        headerRow.appendChild(h);
      });
      usersDiv.appendChild(headerRow);
      const users = (status.users || []).slice().sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { sensitivity: 'base', numeric: true })
      );
      const grid = document.createElement('div');
      grid.style.cssText = 'display:grid;grid-template-columns:repeat(' + serverCount + ', 1fr);gap:6px';
      users.forEach(u => {
        const row = document.createElement('div');
        row.style.cssText = 'display:contents';
        for (let i = 0; i < serverCount; i++) {
          const cell = document.createElement('div');
          const filled = u.servers.includes(i);
          cell.className = 'user-cell' + (filled ? ' filled' : ' empty');
          cell.textContent = filled ? u.name : '·';
          cell.title = filled
            ? (u.servers.length > 1 
                ? u.name + ' is mapped. Watch status changes will mirror in real-time.' 
                : u.name + ' only exists on ' + status.servers[i].name + '. Playback will not sync unless mapped in settings.')
            : (status.servers[i] ? status.servers[i].name + ' has no user named ' + u.name + ' here.' : '');
          row.appendChild(cell);
        }
        grid.appendChild(row);
      });
      usersDiv.appendChild(grid);
      const mappedCount = users.filter(u => u.servers.length > 1).length;
      const singleCount = users.length - mappedCount;
      const legend = document.createElement('div');
      legend.style.cssText = 'margin-top:12px;font-size:11px;color:var(--text);opacity:0.7;display:flex;gap:16px;flex-wrap:wrap';
      legend.innerHTML = '<span>' + users.length + ' users total</span>' +
        '<span style="color:var(--border)">' + mappedCount + ' mapped across servers</span>' +
        '<span style="color:var(--accent)">' + singleCount + ' single-server (need a manual mapping)</span>';
      usersDiv.appendChild(legend);
    }
"#;
