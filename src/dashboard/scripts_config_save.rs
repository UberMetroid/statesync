//! Settings save + connection test handlers.
pub const JS_CONFIG_SAVE: &str = r#"function apiKeyIsPlaceholder(k) {
  k = String(k || '').trim();
  return !k || k.indexOf('•') >= 0 || k.indexOf('*') >= 0;
}
function testConnection() {
  let url = normalizeServerUrl($('serverUrl').value);
  $('serverUrl').value = url;
  const api_key = $('serverKey').value.trim();
  const editing = editIndex >= 0;
  if (!url) return showToast('Enter a server address first');
  if (!editing && apiKeyIsPlaceholder(api_key)) return showToast('Enter an API key first');
  showToast(editing && apiKeyIsPlaceholder(api_key)
    ? 'Testing with saved API key…'
    : 'Testing connection…');
  detectServerType(url, api_key, false, editing ? editIndex : null)
    .then(d => {
      if (d.ok) {
        setDetectedType(d.is_emby, true);
        if (d.url) $('serverUrl').value = d.url;
        showToast(d.message || 'Connected');
      } else {
        showToast(d.message || 'Connection failed');
      }
    })
    .catch((err) => showToast('Connection failed: ' + (err.message || 'unreachable')));
}
$('serverForm').addEventListener('submit', async (e) => {
  e.preventDefault();
  let url = normalizeServerUrl($('serverUrl').value);
  $('serverUrl').value = url;
  const api_key = $('serverKey').value.trim();
  const editing = editIndex >= 0;
  if (!url) return showToast('Enter a server address first');
  if (!editing && apiKeyIsPlaceholder(api_key)) return showToast('Enter an API key first');
  showToast(editing && apiKeyIsPlaceholder(api_key)
    ? 'Checking with saved API key…'
    : 'Detecting server type…');
  let is_emby = $('serverType').value === 'emby';
  try {
    const det = await detectServerType(url, api_key, is_emby, editing ? editIndex : null);
    if (!det.ok) {
      showToast(det.message || 'Could not reach server — fix address/API key before saving');
      return;
    }
    is_emby = det.is_emby;
    if (det.url) { url = det.url; $('serverUrl').value = url; }
    setDetectedType(is_emby, true);
  } catch (err) {
    showToast('Could not detect server type: ' + (err.message || 'unreachable'));
    return;
  }
  // Name is optional — backend fills from hostname if empty
  let name = ($('serverName').value || '').trim();
  if (!name) name = nameFromUrl(url);
  const s = {
    name,
    url,
    api_key,
    is_emby,
    sync_direction: $('serverDirection').value || 'both',
    allow_insecure_http: true
  };
  if (editIndex === -1) { currentConfig.servers.push(s); } else { currentConfig.servers[editIndex] = s; }
  closeModal('serverModal'); await saveConfig();
});
async function deleteServer(idx) {
  const srv = currentConfig.servers[idx];
  const label = srv.name || srv.url || 'this server';
  if (!confirm('Remove ' + label + '?')) return;
  currentConfig.servers.splice(idx, 1);
  await saveConfig();
}
async function saveSettings() {
  currentConfig.sync_threshold_seconds = parseInt($('syncThreshold').value);
  const chk = (id, def) => { const el = $(id); return el ? !!el.checked : def; };
  const allowRaw = ($('cfgUserAllowlist') && $('cfgUserAllowlist').value) || '';
  const user_allowlist = allowRaw.split(/[\n,]+/).map(s => s.trim()).filter(s => s.length > 0);
  const ignRaw = ($('cfgUserIgnorelist') && $('cfgUserIgnorelist').value) || '';
  const user_ignorelist = ignRaw.split(/[\n,]+/).map(s => s.trim()).filter(s => s.length > 0);
  currentConfig.sync = {
    live_played: chk('syncLivePlayed', true),
    live_position: chk('syncLivePosition', true),
    live_favorites: chk('syncLiveFavorites', true),
    force_played: chk('syncForcePlayed', true),
    force_position: chk('syncForcePosition', true),
    force_favorites: chk('syncForceFavorites', true),
    user_allowlist,
    user_ignorelist
  };
  const mappingsLines = $('cfgUserMappings').value.split('\n');
  const user_mappings = [];
  mappingsLines.forEach(line => {
    const parts = line.split(',').map(p => p.trim()).filter(p => p.length > 0);
    if (parts.length > 0) user_mappings.push(parts);
  });
  currentConfig.user_mappings = user_mappings;
  closeModal('settingsModal');
  await saveConfig();
}
async function saveConfig() {
  try {
    const res = await authedFetch('/api/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(currentConfig)
    });
    const body = await res.json();
    showToast(body.message || (res.ok ? 'Saved' : 'Save failed'));
    setTimeout(loadDashboard, 800);
  } catch (err) { showToast('Save failed'); }
}
function showToast(msg) {
  const toast = $('toast');
  toast.innerText = msg;
  toast.style.display = 'block';
  setTimeout(() => { toast.style.display = 'none'; }, 4500);
}
function formatAgo(ms) {
  if (ms < 0) return 'just now';
  const s = Math.floor(ms / 1000);
  if (s < 60) return s + 's ago';
  const m = Math.floor(s / 60);
  if (m < 60) return m + ' min ago';
  const h = Math.floor(m / 60);
  if (h < 24) return h + ' hr ago';
  const d = Math.floor(h / 24);
  return d + ' day' + (d === 1 ? '' : 's') + ' ago';
}
async function refreshUsers() {
  const btn = $('refreshUsersBtn');
  if (btn) btn.disabled = true;
  showToast('Refreshing users…');
  try {
    const res = await authedFetch('/api/users/refresh', { method: 'POST' });
    const data = await res.json();
    showToast('Refreshed ' + ((data.results || []).length) + ' server(s)');
  } catch (err) {
    showToast('Refresh failed: ' + err.message);
  }
  if (btn) btn.disabled = false;
  loadDashboard();
}
"#;
