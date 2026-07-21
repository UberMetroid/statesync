//! Action handlers and log feed rendering for the StateSync web dashboard.

/// Log feed and banner update script string slice (Part 2).
pub const JS_ACTIONS: &str = r#"    const logsDiv = $('syncLogs');
    if (status.sync_logs && status.sync_logs.length > 0) {
      logsDiv.textContent = '';
      status.sync_logs.forEach(log => {
        const line = document.createElement('div'); line.className = 'log-line';
        const prefix = document.createTextNode('> [' + log.timestamp + '] ');
        line.appendChild(prefix);
        if (log.level === 'success' && log.source_name) {
          const sCol = log.source_is_emby ? 'var(--green)' : '#cc00ff';
          const tCol = log.target_is_emby ? 'var(--green)' : '#cc00ff';
          const sBadge = log.source_is_emby ? 'EMBY' : 'JELLYFIN';
          const tBadge = log.target_is_emby ? 'EMBY' : 'JELLYFIN';
          line.appendChild(document.createTextNode(log.message.toUpperCase() + ' FROM '));
          const fromSpan = document.createElement('span'); fromSpan.style.color = sCol;
          fromSpan.textContent = sBadge + ':' + log.source_name;
          line.appendChild(fromSpan);
          line.appendChild(document.createTextNode(' → '));
          const toSpan = document.createElement('span'); toSpan.style.color = tCol;
          toSpan.textContent = tBadge + ':' + log.target_name;
          line.appendChild(toSpan);
        } else {
          const color = log.level === 'error' ? 'var(--red)' : (log.level === 'warn' ? 'var(--accent)' : 'var(--text)');
          const inner = document.createElement('span'); inner.style.color = color;
          inner.textContent = log.level + ': ' + log.message;
          line.appendChild(inner);
        }
        logsDiv.appendChild(line);
      });
      logsDiv.scrollTop = logsDiv.scrollHeight;
    }
    const banner = $('lastFullSyncBanner');
    if (banner && status.last_full_sync) {
      const fs = status.last_full_sync;
      banner.textContent = '';
      const left = document.createElement('span');
      if (fs.finished_at && (fs.state === 'completed' || fs.state === 'failed')) {
        const age = Date.now() - new Date(fs.finished_at).getTime();
        const ago = formatAgo(age);
        const statusColor = fs.state === 'completed' ? 'var(--green)' : 'var(--red)';
        let story = 'Last full sync <span style="color:' + statusColor + '">' + fs.state.toUpperCase() + '</span> ' + ago + '. ';
        story += 'StateSync scanned ' + fs.processed + ' watch history items, successfully aligning ' + fs.succeeded + ' plays';
        if (fs.skipped > 0) story += ', skipping ' + fs.skipped;
        if (fs.failed > 0) story += ', and encountering ' + fs.failed + ' errors';
        story += '.';
        left.innerHTML = story;
        banner.style.borderColor = 'rgba(255,255,255,0.1)';
        banner.style.background = 'rgba(0,0,0,0.2)';
      } else if (fs.started_at) {
        left.innerHTML = 'Full sync in progress · started ' + formatAgo(Date.now() - new Date(fs.started_at).getTime()) + ' ago · ' + fs.processed + ' items so far';
        banner.style.borderColor = 'var(--border)';
        banner.style.background = 'rgba(0,240,255,0.06)';
      } else {
        left.textContent = 'No force sync has been run yet. Click FORCE SYNC to push historical played state across all servers.';
        banner.style.borderColor = 'rgba(255,255,255,0.1)';
        banner.style.background = 'rgba(0,0,0,0.2)';
      }
      banner.appendChild(left);
      const right = document.createElement('span');
      right.style.cssText = 'color:var(--border);cursor:pointer;text-decoration:underline';
      right.textContent = 'run now';
      right.onclick = forceSync;
      banner.appendChild(right);
    }
    const live = $('forceSyncLive');
    if (live) {
      const fs = status.last_full_sync;
      if (fs && fs.state === 'running' && fs.started_at && !fs.finished_at) {
        const totalPairs = fs.total_pairs || 1;
        const processed = fs.processed || 0;
        const pct = Math.min(100, Math.floor(processed / totalPairs * 100));
        const elapsed = Math.max(1, Math.round((Date.now() - new Date(fs.started_at).getTime()) / 1000));
        const rate = elapsed > 0 ? (processed / elapsed).toFixed(1) : '0';
        live.style.display = 'block';
        const bar = $('fsProgressBar');
        if (bar) { bar.value = pct; bar.max = 100; }
        const txt = $('fsProgressText');
        if (txt) txt.textContent = pct + '% · ' + processed + ' / ' + totalPairs + ' items (' + rate + '/s · ' + formatAgo(elapsed * 1000) + ')';
        const cu = $('fsCurrentUser');
        if (cu) cu.textContent = fs.current_user ? 'currently syncing: ' + fs.current_user : '';
      } else {
        live.style.display = 'none';
      }
    }
    const forceBtn = $('forceSyncBtn');
    if (forceBtn) {
      const noServers = !currentConfig.servers || currentConfig.servers.length === 0;
      const inProgress = status.last_full_sync &&
                        (status.last_full_sync.state === 'running' || (status.last_full_sync.started_at && !status.last_full_sync.finished_at));
      forceBtn.disabled = noServers || inProgress;
      if (noServers) {
        forceBtn.title = 'Add a media server first';
      } else if (inProgress) {
        forceBtn.title = 'A force sync is already running';
      } else {
        forceBtn.removeAttribute('title');
      }
    }
    const footer = $('versionFooter');
    if (footer && status.version) {
      footer.textContent = '';
      const link = document.createElement('a');
      link.href = 'https://github.com/studio2201/statesync/releases/tag/v' + status.version;
      link.target = '_blank';
      link.rel = 'noopener noreferrer';
      link.textContent = 'v' + status.version;
      link.style.cssText = 'color: var(--accent); text-decoration: none; border-bottom: 1px dotted var(--accent);';
      footer.appendChild(link);
      footer.appendChild(document.createTextNode(' | uptime ' + Math.floor(status.uptime_seconds / 60) + 'm'));
    }
  } catch (err) { console.error(err); }
}
"#;
