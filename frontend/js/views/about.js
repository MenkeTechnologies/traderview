// About — exhaustive identity card. Every number on the page is
// derived from a live registry, the DOM, localStorage, or the running
// Tauri process. Hardcoded constants stay out — they drift on every
// feature add.

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderAbout(mount, _state) {
    const [tiles, vr, sc, ctx, apiMod] = await Promise.all([
        import('./launcher.js').then(m => m.TILES || []),
        import('../app.js').then(m => m.viewRenderers || {}),
        import('../_shortcuts.js').then(m => m.DEFAULT_SHORTCUTS || []),
        import('../_context_menu.js'),
        import('../api.js'),
    ]);

    const cats = await import('./launcher.js').then(m => m.CATEGORIES || []);

    const ver = (typeof window !== 'undefined' && window.__TRADERVIEW_VERSION__) || 'dev';
    const locales = ['en','cs','da','de','el','es','es_419','fi','fr','hi','hu','id','it','ja','ko','nb','nl','pl','pt','pt_br','ro','ru','sv','tr','uk','vi','zh'];

    // ── frontend registry counts ────────────────────────────────────────
    const tileCount = tiles.length;
    const vrCount = Object.keys(vr).length;
    const shortcutCount = sc.length;
    const symScopeCount = (ctx.SYMBOL_AWARE_SCOPES || []).length;
    const rowScopeCount = (ctx.ALL_SCOPED_ITEMS || []).length;
    const ctxItemCount = (ctx.ALL_SCOPED_ITEMS || [])
        .reduce((acc, [, items]) => acc + items.length, 0)
        + (ctx.GLOBAL_ITEMS || []).length
        + (ctx.SYMBOL_ITEMS || []).length;
    const apiMethodCount = Object.keys(apiMod.api || {}).length;
    const tilesWithBadge = tiles.filter(t => t[4]).length;
    const tileGlyphs = new Set(tiles.map(t => t[2])).size;

    // ── shortcut breakdown ──────────────────────────────────────────────
    const scopeCounts = {};
    sc.forEach(s => { scopeCounts[s.scope || 'global'] = (scopeCounts[s.scope || 'global'] || 0) + 1; });
    const boundShortcuts = sc.filter(s => s.keys && s.keys.key).length;

    // ── category breakdown (tiles per category) ─────────────────────────
    const catRows = cats.map(([id, label, ids]) =>
        `<tr><td>${esc(label.replace(/^\/\/\s*/, ''))}</td>
             <td class="muted small">${esc(id)}</td>
             <td class="num">${ids.length}</td></tr>`
    ).join('');

    // ── localStorage state ──────────────────────────────────────────────
    const ls = readLocalStorageStats();

    // ── runtime / browser ───────────────────────────────────────────────
    const nav = (typeof navigator !== 'undefined') ? navigator : {};
    const screen = (typeof window !== 'undefined') ? window.screen || {} : {};
    const isDesktop = typeof window !== 'undefined'
        && !!window.__TAURI__
        && !!window.__TAURI__.core
        && typeof window.__TAURI__.core.invoke === 'function';

    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.about.title">// ABOUT TRADERVIEW</span>
            ${isDesktop ? `<button class="btn btn-sm btn-secondary" id="about-refresh" style="margin-left:12px"
                    data-tip="view.about.tip.refresh">⟳ Refresh</button>` : ''}
        </h1>
        <p class="muted small" data-i18n="view.about.intro">
            Self-hosted trading + personal-finance HUD. Tauri desktop shell (Rust + WebKit),
            Postgres-backed analytics, real-time WS tape, multi-broker order routing,
            150+ technical indicators, full tax-code calculators, AI-augmented journal.
        </p>

        <!-- ── top-line stat cards ───────────────────────────────────── -->
        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:8px;margin-bottom:16px">
            <div class="card"><div class="label">Version</div><div class="value">${esc(ver)}</div></div>
            <div class="card"><div class="label">Mode</div><div class="value">${isDesktop ? 'desktop' : 'web'}</div></div>
            <div class="card"><div class="label">Tiles</div><div class="value">${tileCount}</div></div>
            <div class="card"><div class="label">Renderers</div><div class="value">${vrCount}</div></div>
            <div class="card"><div class="label">Categories</div><div class="value">${cats.length}</div></div>
            <div class="card"><div class="label">Shortcuts</div><div class="value">${shortcutCount}</div></div>
            <div class="card"><div class="label">Bound keys</div><div class="value">${boundShortcuts}</div></div>
            <div class="card"><div class="label">API methods</div><div class="value">${apiMethodCount}</div></div>
            <div class="card"><div class="label">Locales</div><div class="value">${locales.length}</div></div>
            <div class="card"><div class="label">Sym scopes</div><div class="value">${symScopeCount}</div></div>
            <div class="card"><div class="label">Row scopes</div><div class="value">${rowScopeCount}</div></div>
            <div class="card"><div class="label">Ctx items</div><div class="value">${ctxItemCount}</div></div>
        </div>

        ${isDesktop ? `
        <!-- ── process / hardware (desktop only) ─────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.process">Process · hardware</h2>
            <table class="trades" data-table-key="about-process" id="about-process-table">
                <thead><tr><th>Stat</th><th>Value</th></tr></thead>
                <tbody><tr><td colspan="2" class="muted">loading…</td></tr></tbody>
            </table>
        </div>` : ''}

        <!-- ── persisted state ──────────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.persisted">Persisted state · this browser</h2>
            <table class="trades" data-table-key="about-persisted">
                <thead><tr><th>Item</th><th>Count / size</th></tr></thead>
                <tbody>
                    <tr><td>Dashboards</td><td class="num">${ls.dashboards}</td></tr>
                    <tr><td>Dashboard tiles (total)</td><td class="num">${ls.dashTilesTotal}</td></tr>
                    <tr><td>Active dashboard</td><td>${esc(ls.activeDashboard || '—')}</td></tr>
                    <tr><td>Favorites</td><td class="num">${ls.favorites}</td></tr>
                    <tr><td>Bookmarks</td><td class="num">${ls.bookmarks}</td></tr>
                    <tr><td>Recently-visited views</td><td class="num">${ls.recents}</td></tr>
                    <tr><td>Custom column widths</td><td class="num">${ls.tableCols}</td></tr>
                    <tr><td>Saved table sorts</td><td class="num">${ls.tableSorts}</td></tr>
                    <tr><td>localStorage keys</td><td class="num">${ls.totalKeys}</td></tr>
                    <tr><td>localStorage size</td><td>${fmtBytes(ls.totalBytes)}</td></tr>
                    <tr><td>Sidebar collapsed</td><td>${ls.sidebarCollapsed ? 'yes' : 'no'}</td></tr>
                </tbody>
            </table>
        </div>

        <!-- ── category breakdown ───────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.feature_breakdown">Feature breakdown</h2>
            <table class="trades" data-table-key="about-categories">
                <thead><tr><th>Category</th><th>ID</th><th>Tiles</th></tr></thead>
                <tbody>${catRows}</tbody>
            </table>
            <p class="muted small" style="margin-top:8px">
                ${tilesWithBadge} of ${tileCount} tiles carry a status badge (NEW / LIVE / β).
                ${tileGlyphs} distinct tile glyphs in use.
            </p>
        </div>

        <!-- ── shortcut breakdown ───────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.shortcuts">Keyboard shortcut scopes</h2>
            <table class="trades" data-table-key="about-shortcut-scopes">
                <thead><tr><th>Scope</th><th>Count</th></tr></thead>
                <tbody>${Object.entries(scopeCounts)
                    .sort((a, b) => b[1] - a[1])
                    .map(([s, n]) => `<tr><td>${esc(s)}</td><td class="num">${n}</td></tr>`).join('')}</tbody>
            </table>
            <p class="muted small" style="margin-top:8px">
                ${boundShortcuts} of ${shortcutCount} shortcuts have a default key binding;
                the rest are palette / context-menu only.
            </p>
        </div>

        <!-- ── runtime environment ──────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.runtime">Runtime · browser</h2>
            <table class="trades" data-table-key="about-runtime">
                <thead><tr><th>Stat</th><th>Value</th></tr></thead>
                <tbody>
                    <tr><td>User-Agent</td><td class="muted small" style="word-break:break-all">${esc(nav.userAgent || '—')}</td></tr>
                    <tr><td>Platform</td><td>${esc(nav.platform || '—')}</td></tr>
                    <tr><td>Language</td><td>${esc(nav.language || '—')}</td></tr>
                    <tr><td>Hardware threads</td><td class="num">${nav.hardwareConcurrency || '—'}</td></tr>
                    <tr><td>Device memory</td><td>${nav.deviceMemory ? nav.deviceMemory + ' GiB' : '—'}</td></tr>
                    <tr><td>Online</td><td>${nav.onLine === false ? '<span class="neg">offline</span>' : 'yes'}</td></tr>
                    <tr><td>Screen</td><td>${screen.width || '?'} × ${screen.height || '?'} (${screen.colorDepth || '?'}-bit)</td></tr>
                    <tr><td>Pixel ratio</td><td>${(window.devicePixelRatio || 1).toFixed(2)}</td></tr>
                    <tr><td>Timezone</td><td>${esc(Intl.DateTimeFormat().resolvedOptions().timeZone || '—')}</td></tr>
                    <tr><td>Window size</td><td>${window.innerWidth} × ${window.innerHeight}</td></tr>
                    <tr><td>Document URL</td><td class="muted small" style="word-break:break-all">${esc(window.location.href)}</td></tr>
                </tbody>
            </table>
        </div>

        <!-- ── tech stack ───────────────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.stack">Tech stack</h2>
            <table class="trades" data-table-key="about-stack">
                <thead><tr><th>Layer</th><th>Tech</th></tr></thead>
                <tbody>
                    <tr><td>Desktop shell</td><td>Tauri 2.x (Rust + WebKit)</td></tr>
                    <tr><td>Backend HTTP</td><td>axum + tokio (Rust)</td></tr>
                    <tr><td>Database</td><td>PostgreSQL (sqlx, embedded postgresql_embedded)</td></tr>
                    <tr><td>Live ticks</td><td>WebSocket — Alpaca / Polygon / Finnhub cascade</td></tr>
                    <tr><td>Charts</td><td>uPlot (canvas) + TradingView lightweight</td></tr>
                    <tr><td>Frontend</td><td>Vanilla ES modules — no React, no build step in dev</td></tr>
                    <tr><td>i18n</td><td>LibreTranslate (self-hosted) + Google Translate fallback</td></tr>
                    <tr><td>AI Journal</td><td>Anthropic / OpenAI / Ollama (configurable)</td></tr>
                    <tr><td>Tables</td><td>frontend/js/table_enhance.js — auto-sort + col-resize</td></tr>
                    <tr><td>Filtering</td><td>fzf-style fuzzy + highlight (frontend/js/fzf.js)</td></tr>
                    <tr><td>Drag-reorder</td><td>pointer-driven (works through canvases)</td></tr>
                </tbody>
            </table>
        </div>

        <!-- ── data providers ───────────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.providers">Data providers · all keys in Settings → Data Sources</h2>
            <table class="trades" data-table-key="about-providers">
                <thead><tr><th>Provider</th><th>Use</th><th>Tier</th></tr></thead>
                <tbody>
                    <tr><td>Finnhub</td><td>Quote, news, fundamentals, recommendations, halts</td><td>Free 60/min</td></tr>
                    <tr><td>Alpaca</td><td>Live ticks (IEX or SIP), paper + live order routing</td><td>Free IEX / $99 SIP</td></tr>
                    <tr><td>Polygon</td><td>Full CTA/UTP SIP tape, options chain history</td><td>$29+ tier</td></tr>
                    <tr><td>Databento</td><td>Direct CTA / UTP / OPRA feeds</td><td>Pay-per-byte</td></tr>
                    <tr><td>Tradier</td><td>Brokerage execution</td><td>Sandbox + live</td></tr>
                    <tr><td>Tastytrade</td><td>Brokerage execution</td><td>Sandbox + live</td></tr>
                    <tr><td>IBKR</td><td>Brokerage execution (Client Portal API)</td><td>Self-hosted gateway</td></tr>
                    <tr><td>Schwab</td><td>Brokerage execution (Trader API)</td><td>OAuth, free</td></tr>
                    <tr><td>Webull</td><td>Read-only positions / orders (browser session tokens)</td><td>Free, no API</td></tr>
                    <tr><td>Yahoo Finance</td><td>Historical bars (1m → 1d), quote summary</td><td>Anonymous</td></tr>
                    <tr><td>Anthropic / OpenAI / Ollama</td><td>AI journal analysis</td><td>Per-call / local</td></tr>
                </tbody>
            </table>
        </div>

        <!-- ── locale catalog ──────────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.locales">Locale catalog (${locales.length} locales)</h2>
            <p class="muted small" style="margin-bottom:8px">
                Active: <strong>${esc(localStorage.getItem('tv-locale-v1') || 'en')}</strong>.
                Translations served via LibreTranslate at the user's local instance + Google Translate fallback.
                Per-key proper-noun blacklist masks brand names / acronyms so they survive translation.
            </p>
            <code class="muted small" style="word-break:break-word">${locales.join(' · ')}</code>
        </div>

        <!-- ── links ───────────────────────────────────────────────── -->
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.links">Links</h2>
            <p class="muted small">
                <a class="link" href="https://github.com/MenkeTechnologies/traderview" target="_blank" rel="noopener">GitHub</a>
                ·
                <a class="link" href="https://github.com/MenkeTechnologies/traderview/releases" target="_blank" rel="noopener">Releases</a>
                ·
                <a class="link" href="https://github.com/MenkeTechnologies/traderview/blob/main/LICENSE" target="_blank" rel="noopener">License</a>
                ·
                <a class="link" href="https://github.com/MenkeTechnologies/traderview/issues" target="_blank" rel="noopener">Issues</a>
                ·
                <a class="link" href="https://github.com/MenkeTechnologies/zpwr" target="_blank" rel="noopener">zpwr</a>
                ·
                <a class="link" href="https://github.com/MenkeTechnologies/zsh-more-completions" target="_blank" rel="noopener">zsh-more-completions</a>
            </p>
        </div>
    `;

    if (isDesktop) {
        const refreshBtn = mount.querySelector('#about-refresh');
        const refresh = () => paintProcessStats(mount);
        if (refreshBtn) refreshBtn.addEventListener('click', refresh);
        refresh();
    }
}

// ── helpers ──────────────────────────────────────────────────────────

function fmtBytes(b) {
    if (!Number.isFinite(b) || b <= 0) return '—';
    const u = ['B','KiB','MiB','GiB','TiB'];
    let i = 0, v = b;
    while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
    return `${v.toFixed(v < 10 ? 2 : v < 100 ? 1 : 0)} ${u[i]}`;
}

function fmtUptime(sec) {
    if (!Number.isFinite(sec) || sec < 0) return '—';
    const d = Math.floor(sec / 86400);
    const h = Math.floor((sec % 86400) / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const ss = Math.floor(sec % 60);
    const parts = [];
    if (d) parts.push(`${d}d`);
    if (h || d) parts.push(`${h}h`);
    if (m || h || d) parts.push(`${m}m`);
    parts.push(`${ss}s`);
    return parts.join(' ');
}

// Walk every localStorage key, sum sizes, count specific known buckets.
function readLocalStorageStats() {
    const out = {
        dashboards: 0, dashTilesTotal: 0, activeDashboard: null,
        favorites: 0, bookmarks: 0, recents: 0,
        tableCols: 0, tableSorts: 0,
        totalKeys: 0, totalBytes: 0,
        sidebarCollapsed: false,
    };
    try {
        for (let i = 0; i < localStorage.length; i++) {
            const k = localStorage.key(i);
            if (k == null) continue;
            const v = localStorage.getItem(k) || '';
            out.totalKeys++;
            out.totalBytes += k.length + v.length;
            if (k.startsWith('tv:tableCols:')) out.tableCols++;
            else if (k.startsWith('tv:tableSort:')) out.tableSorts++;
            else if (k.startsWith('tv:launcherOrder:')) { /* count not surfaced */ }
            else if (k === 'tv:dashSidebarCollapsed') out.sidebarCollapsed = v === '1';
        }
        // Dashboards / favorites / bookmarks / recents have known schemas.
        const dashRaw = localStorage.getItem('tv:dashboards-v1') || localStorage.getItem('tv-dashboards');
        if (dashRaw) {
            try {
                const d = JSON.parse(dashRaw);
                if (d && d.dashboards) {
                    out.dashboards = Object.keys(d.dashboards).length;
                    out.dashTilesTotal = Object.values(d.dashboards)
                        .reduce((acc, x) => acc + (Array.isArray(x.tiles) ? x.tiles.length : 0), 0);
                    out.activeDashboard = d.active || null;
                }
            } catch {}
        }
        const favRaw = localStorage.getItem('tv:favorites-v1') || localStorage.getItem('tv-favorites');
        if (favRaw) {
            try {
                const f = JSON.parse(favRaw);
                if (f) {
                    out.favorites = Array.isArray(f.favorites) ? f.favorites.length : 0;
                    out.bookmarks = Array.isArray(f.bookmarks) ? f.bookmarks.length : 0;
                }
            } catch {}
        }
        const recRaw = localStorage.getItem('tv:recents-v1') || localStorage.getItem('tv-recents');
        if (recRaw) {
            try {
                const r = JSON.parse(recRaw);
                if (r && Array.isArray(r.recents)) out.recents = r.recents.length;
            } catch {}
        }
    } catch {}
    return out;
}

// Pull a fresh snapshot from the Tauri `get_process_stats` command and
// re-render the Process table. Falls back to a "web-only" notice when
// running outside the Tauri shell (browser dev).
async function paintProcessStats(mount) {
    const tbody = mount.querySelector('#about-process-table tbody');
    if (!tbody) return;
    const tauri = typeof window !== 'undefined' && window.__TAURI__;
    if (!tauri || !tauri.core || typeof tauri.core.invoke !== 'function') {
        tbody.innerHTML = `<tr><td colspan="2" class="muted">${esc(t('view.about.process.web_only') || 'Process stats are only available in the desktop app.')}</td></tr>`;
        return;
    }
    let s;
    try { s = await tauri.core.invoke('get_process_stats'); }
    catch (e) {
        tbody.innerHTML = `<tr><td colspan="2" class="neg">${esc(String(e))}</td></tr>`;
        return;
    }
    const cpuPctOfHost = s.cpus > 0 ? (s.cpu_percent / s.cpus) : 0;
    const memPctOfHost = s.total_memory_bytes > 0 ? (s.rss_bytes / s.total_memory_bytes * 100) : 0;
    const hostMemUsed = s.total_memory_bytes - s.available_memory_bytes;
    const hostMemPct = s.total_memory_bytes > 0 ? (hostMemUsed / s.total_memory_bytes * 100) : 0;
    const swapPct = s.total_swap_bytes > 0 ? (s.used_swap_bytes / s.total_swap_bytes * 100) : 0;
    const diskUsed = s.disk_total_bytes - s.disk_free_bytes;
    const diskPct = s.disk_total_bytes > 0 ? (diskUsed / s.disk_total_bytes * 100) : 0;
    const bootDate = s.boot_time_secs ? new Date(s.boot_time_secs * 1000).toLocaleString() : '—';
    const rows = [
        // ── identity ────────────────────────────────────────────────
        ['PID', String(s.pid)],
        ['App version', `v${s.app_version}`],
        ['Hostname', s.hostname],
        ['OS', s.os_version || s.os],
        ['Kernel', s.kernel_version || '—'],
        ['Architecture', s.arch],
        // ── CPU ─────────────────────────────────────────────────────
        ['CPU', s.cpu_brand || '—'],
        ['CPU cores', String(s.cpus)],
        ['CPU frequency', s.cpu_frequency_mhz > 0 ? `${(s.cpu_frequency_mhz / 1000).toFixed(2)} GHz` : '—'],
        ['Process CPU %', `${s.cpu_percent.toFixed(1)}% (${cpuPctOfHost.toFixed(2)}% of host)`],
        ['Load average', `${s.load_avg_1.toFixed(2)}, ${s.load_avg_5.toFixed(2)}, ${s.load_avg_15.toFixed(2)} (1m, 5m, 15m)`],
        // ── memory ──────────────────────────────────────────────────
        ['Resident (RSS)', `${fmtBytes(s.rss_bytes)} (${memPctOfHost.toFixed(2)}% of host)`],
        ['Virtual memory', fmtBytes(s.virtual_bytes)],
        s.threads ? ['Threads', String(s.threads)] : null,
        ['Host RAM total', fmtBytes(s.total_memory_bytes)],
        ['Host RAM used', `${fmtBytes(hostMemUsed)} (${hostMemPct.toFixed(1)}%)`],
        ['Host RAM free', fmtBytes(s.available_memory_bytes)],
        s.total_swap_bytes > 0
            ? ['Swap', `${fmtBytes(s.used_swap_bytes)} used / ${fmtBytes(s.total_swap_bytes)} (${swapPct.toFixed(1)}%)`]
            : null,
        // ── disk (root volume) ──────────────────────────────────────
        s.disk_total_bytes > 0
            ? ['Disk (root)', `${fmtBytes(diskUsed)} used / ${fmtBytes(s.disk_total_bytes)} (${diskPct.toFixed(1)}%)`]
            : null,
        // ── time ────────────────────────────────────────────────────
        ['Process uptime', fmtUptime(s.uptime_secs)],
        ['System uptime', fmtUptime(s.system_uptime_secs)],
        ['Booted', bootDate],
        ['Snapshot taken', new Date().toLocaleString()],
    ].filter(Boolean);
    tbody.innerHTML = rows.map(([k, v]) =>
        `<tr><td class="muted small">${esc(k)}</td><td><strong>${esc(v)}</strong></td></tr>`
    ).join('');
}
