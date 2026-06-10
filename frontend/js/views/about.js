// About — single-source-of-truth identity card. Pulls live counts from
// the in-process registries (TILES, viewRenderers, DEFAULT_SHORTCUTS,
// LOCALE_TO_LT, SYMBOL_AWARE_SCOPES, ALL_SCOPED_ITEMS) rather than
// hardcoding numbers that would drift on every feature add.

import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderAbout(mount, _state) {
    // Pull every registry lazily so this view doesn't pin them via the
    // static-import graph (and so it survives an early-boot mount before
    // app.js's exports have settled).
    const [tiles, vr, sc, ctx] = await Promise.all([
        import('./launcher.js').then(m => m.TILES || []),
        import('../app.js').then(m => m.viewRenderers || {}),
        import('../_shortcuts.js').then(m => m.DEFAULT_SHORTCUTS || []),
        import('../_context_menu.js').then(m => ({
            sym: m.SYMBOL_AWARE_SCOPES || [],
            row: m.ALL_SCOPED_ITEMS || [],
        })),
    ]);

    const ver = (typeof window !== 'undefined' && window.__TRADERVIEW_VERSION__) || 'dev';
    const locales = ['en','cs','da','de','el','es','es_419','fi','fr','hi','hu','id','it','ja','ko','nb','nl','pl','pt','pt_br','ro','ru','sv','tr','uk','vi','zh'];
    // Hardware / process stats only make sense in the desktop app — the
    // browser sandbox can't see PID, RSS, host RAM, or other system info.
    // Hide the whole panel in web deployments.
    const isDesktop = typeof window !== 'undefined'
        && !!window.__TAURI__
        && !!window.__TAURI__.core
        && typeof window.__TAURI__.core.invoke === 'function';

    const tileCount = Array.isArray(tiles) ? tiles.length : 0;
    const vrCount = Object.keys(vr).length;
    const shortcutCount = sc.length;
    const symScopeCount = ctx.sym.length;
    const rowScopeCount = ctx.row.length;
    const localeCount = locales.length;

    // Category breakdown — count tiles per category by reading CATEGORIES.
    const cats = await import('./launcher.js').then(m => m.CATEGORIES || []);
    const catRows = cats.map(([id, label, ids]) =>
        `<tr><td>${esc(label.replace(/^\/\/\s*/, ''))}</td><td class="num">${ids.length}</td></tr>`
    ).join('');

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

        <div class="cards" style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:10px;margin-bottom:16px">
            <div class="card"><div class="label">Version</div><div class="value">${esc(ver)}</div></div>
            <div class="card"><div class="label">Tiles</div><div class="value">${tileCount}</div></div>
            <div class="card"><div class="label">Renderers</div><div class="value">${vrCount}</div></div>
            <div class="card"><div class="label">Shortcuts</div><div class="value">${shortcutCount}</div></div>
            <div class="card"><div class="label">Locales</div><div class="value">${localeCount}</div></div>
            <div class="card"><div class="label">Ctx scopes</div><div class="value">${symScopeCount + rowScopeCount}</div></div>
        </div>

        ${isDesktop ? `
        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.process">Process</h2>
            <table class="trades" data-table-key="about-process" id="about-process-table">
                <thead><tr><th>Stat</th><th>Value</th></tr></thead>
                <tbody><tr><td colspan="2" class="muted">loading…</td></tr></tbody>
            </table>
        </div>
        ` : ''}

        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.feature_breakdown">Feature breakdown by category</h2>
            <table class="trades" data-table-key="about-categories">
                <thead><tr><th data-i18n="view.about.th.category">Category</th><th data-i18n="view.about.th.tiles">Tiles</th></tr></thead>
                <tbody>${catRows}</tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.stack">Tech stack</h2>
            <table class="trades" data-table-key="about-stack">
                <thead><tr><th data-i18n="view.about.th.layer">Layer</th><th data-i18n="view.about.th.tech">Tech</th></tr></thead>
                <tbody>
                    <tr><td>Desktop shell</td><td>Tauri 2.x (Rust + WebKit)</td></tr>
                    <tr><td>Backend HTTP</td><td>axum + tokio (Rust)</td></tr>
                    <tr><td>Database</td><td>PostgreSQL (sqlx)</td></tr>
                    <tr><td>Live ticks</td><td>WebSocket — Alpaca / Polygon / Finnhub cascade</td></tr>
                    <tr><td>Charts</td><td>uPlot (canvas), TradingView lightweight</td></tr>
                    <tr><td>Frontend</td><td>Vanilla ES modules — no React, no build step in dev</td></tr>
                    <tr><td>i18n</td><td>LibreTranslate (self-hosted) + Google Translate fallback</td></tr>
                    <tr><td>AI Journal</td><td>Anthropic / OpenAI / Ollama (configurable)</td></tr>
                </tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.providers">Data providers</h2>
            <p class="muted small" data-i18n="view.about.providers.intro">
                All provider keys live in Settings → Data Sources. Free tiers work for most
                features; paid tiers unlock SIP tape and faster polling.
            </p>
            <table class="trades" data-table-key="about-providers">
                <thead><tr><th data-i18n="view.about.th.provider">Provider</th><th data-i18n="view.about.th.use">Use</th><th data-i18n="view.about.th.tier">Tier</th></tr></thead>
                <tbody>
                    <tr><td>Finnhub</td><td>Quote, news, fundamentals, recommendations, halts</td><td>Free 60/min</td></tr>
                    <tr><td>Alpaca</td><td>Live ticks (IEX or SIP), paper + live order routing</td><td>Free IEX / $99 SIP</td></tr>
                    <tr><td>Polygon</td><td>Full CTA/UTP SIP tape, options chain history</td><td>$29+ tier</td></tr>
                    <tr><td>Databento</td><td>Direct CTA / UTP / OPRA feeds</td><td>Pay-per-byte</td></tr>
                    <tr><td>Tradier</td><td>Brokerage execution</td><td>Sandbox + live</td></tr>
                    <tr><td>Tastytrade</td><td>Brokerage execution</td><td>Sandbox + live</td></tr>
                    <tr><td>IBKR</td><td>Brokerage execution (Client Portal API)</td><td>Self-hosted gateway</td></tr>
                    <tr><td>Schwab</td><td>Brokerage execution (Trader API)</td><td>OAuth, free</td></tr>
                    <tr><td>Yahoo Finance</td><td>Historical bars (1m → 1d), quote summary</td><td>Anonymous</td></tr>
                </tbody>
            </table>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.about.h2.links">Links</h2>
            <p class="muted small">
                <a class="link" href="https://github.com/MenkeTechnologies/traderview" target="_blank" rel="noopener">GitHub</a>
                ·
                <a class="link" href="https://github.com/MenkeTechnologies/traderview/releases" target="_blank" rel="noopener">Releases</a>
                ·
                <a class="link" href="https://github.com/MenkeTechnologies/traderview/blob/main/LICENSE" target="_blank" rel="noopener">License</a>
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
    const fmtBytes = (b) => {
        if (!Number.isFinite(b) || b <= 0) return '—';
        const u = ['B','KiB','MiB','GiB','TiB'];
        let i = 0, v = b;
        while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
        return `${v.toFixed(v < 10 ? 2 : v < 100 ? 1 : 0)} ${u[i]}`;
    };
    const fmtUptime = (sec) => {
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
    };
    const rows = [
        ['PID', String(s.pid)],
        ['Version', `v${s.app_version}`],
        ['Hostname', s.hostname],
        ['OS / arch', `${s.os} · ${s.arch}`],
        ['CPUs', `${s.cpus} (${s.cpu_percent.toFixed(1)}%)`],
        ['Resident (RSS)', fmtBytes(s.rss_bytes)],
        ['Virtual', fmtBytes(s.virtual_bytes)],
        s.threads ? ['Threads', String(s.threads)] : null,
        ['Uptime', fmtUptime(s.uptime_secs)],
        ['Host memory', `${fmtBytes(s.available_memory_bytes)} free / ${fmtBytes(s.total_memory_bytes)} total`],
    ].filter(Boolean);
    tbody.innerHTML = rows.map(([k, v]) =>
        `<tr><td class="muted small">${esc(k)}</td><td><strong>${esc(v)}</strong></td></tr>`
    ).join('');
}
