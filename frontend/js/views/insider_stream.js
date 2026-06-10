// Insider Form 4 real-time stream — fetches every SEC Form 4 filing
// (insider transaction) off the EDGAR catalyst stream, parses the
// XML for insider name/title/transaction code/shares/price, and
// surfaces buys, sales, grants, and option exercises separately.
// TTS on every Buy ≥ $1M from an officer.

import { api, wsUrl } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';

let ws = null;
let viewTok = 0;
let voiceOn = true;
let kindFilter = 'buy'; // 'buy' | 'sell' | 'all'
let minDollar = 100_000;
const rows = new Map();

export async function renderInsiderStream(mount, _state) {
    viewTok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.insider_stream.title">// INSIDER FORM 4 STREAM · LIVE</span>
            <span class="status-dot" id="is-status" data-i18n-title="common.status.connecting" title="connecting">●</span>
            <label class="is-voice-toggle">
                <input type="checkbox" id="is-voice" ${voiceOn ? 'checked' : ''}>
                <span data-i18n="common.label.voice_alerts">voice alerts</span>
            </label>
        </h1>
        <p class="muted small" data-i18n-html="view.insider_stream.intro">
            Subscribes to the SEC EDGAR catalyst stream, filters to Form 4 filings,
            fetches each one's XML, and extracts insider name, title (officer/director/
            10%-owner), transaction code, shares, and price-per-share. Cluster insider
            buys are the academically-best free signal — Lakonishok &amp; Lee 2001
            found ~5-10% annual outperformance over the next 6-12 months. TTS fires
            on every officer Buy ≥ $1M.
        </p>
        <div class="chart-panel">
            <div class="is-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <div class="is-toggle" role="tablist" aria-label="kind">
                    <button class="btn btn-sm" data-kind="buy"  data-i18n="view.insider_stream.btn.buys">Buys</button>
                    <button class="btn btn-sm" data-kind="sell" data-i18n="view.insider_stream.btn.sales">Sales</button>
                    <button class="btn btn-sm" data-kind="all"  data-i18n="view.insider_stream.btn.all">All</button>
                </div>
                <label class="is-min-dollar">
                    <span data-i18n="view.insider_stream.label.min_dollar">min $</span>
                    <input type="number" id="is-min-dollar" min="0" step="50000" value="${minDollar}" style="width:120px">
                </label>
                <button class="btn btn-sm" id="is-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <button class="btn btn-sm" id="is-top-buys" data-i18n="view.insider_stream.btn.top_buys">Top 30d Buys</button>
            </div>
            <div id="is-top-buys-panel" style="display:none;margin-bottom:12px"></div>
            <table class="trades" id="is-table">
                <thead><tr>
                    <th data-i18n="view.insider_stream.th.observed">Observed</th>
                    <th data-i18n="view.insider_stream.th.symbol">Symbol</th>
                    <th data-i18n="view.insider_stream.th.insider">Insider</th>
                    <th data-i18n="view.insider_stream.th.role">Role</th>
                    <th data-i18n="view.insider_stream.th.kind">Type</th>
                    <th data-i18n="view.insider_stream.th.shares">Shares</th>
                    <th data-i18n="view.insider_stream.th.price">Price</th>
                    <th data-i18n="view.insider_stream.th.dollars">Dollars</th>
                    <th data-i18n="view.insider_stream.th.tx_date">Tx Date</th>
                </tr></thead>
                <tbody><tr><td colspan="9" class="muted" data-i18n="common.connecting">connecting…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#is-voice').addEventListener('change', (e) => { voiceOn = e.target.checked; });
    mount.querySelector('#is-min-dollar').addEventListener('change', (e) => {
        const v = parseFloat(e.target.value);
        if (Number.isFinite(v) && v >= 0) { minDollar = v; render(); }
    });
    mount.querySelectorAll('[data-kind]').forEach(b => {
        b.addEventListener('click', () => { kindFilter = b.dataset.kind; applyToggleState(mount); render(); });
    });
    mount.querySelector('#is-refresh').addEventListener('click', () => {
        rows.clear();
        connectWs(mount, viewTok);
    });
    mount.querySelector('#is-top-buys').addEventListener('click', () => renderTopBuys(mount));
    applyToggleState(mount);
    connectWs(mount, viewTok);
}

function applyToggleState(mount) {
    mount.querySelectorAll('[data-kind]').forEach(b => {
        b.classList.toggle('active', b.dataset.kind === kindFilter);
    });
}

async function renderTopBuys(mount) {
    const panel = mount.querySelector('#is-top-buys-panel');
    if (!panel) return;
    panel.style.display = '';
    panel.innerHTML = `<div class="muted small">${esc(t('common.loading'))}</div>`;
    try {
        const rows = await api.request('/insider-stream/top-buys?days=30&limit=25');
        if (!rows.length) {
            panel.innerHTML = `<div class="muted small">${esc(t('view.insider_stream.empty.no_top_buys'))}</div>`;
            return;
        }
        panel.innerHTML = `
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.insider_stream.th.symbol">Symbol</th>
                    <th data-i18n="view.insider_stream.th.total_dollars">Total Buys (30d)</th>
                    <th data-i18n="view.insider_stream.th.tx_count">Tx Count</th>
                </tr></thead>
                <tbody>${rows.map(r => `
                    <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                        <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                        <td class="pos">${fmtDollar(r.total_dollars)}</td>
                        <td>${r.transaction_count}</td>
                    </tr>`).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        panel.innerHTML = `<div class="muted small">${esc(String(e))}</div>`;
    }
}

function connectWs(mount, tok) {
    try { if (ws) { try { ws.close(); } catch (_) {} ws = null; } } catch (_) {}
    if (!viewIsCurrent(tok)) return;
    const dot = mount.querySelector('#is-status');
    if (!dot) return;
    ws = new WebSocket(wsUrl('/api/ws/insider-stream'));
    ws.addEventListener('open',  () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--green)'; dot.title = t('common.status.connected'); } });
    ws.addEventListener('error', () => { if (viewIsCurrent(tok)) { dot.style.color = 'var(--red)';   dot.title = t('common.status.error'); } });
    ws.addEventListener('close', () => {
        if (!viewIsCurrent(tok)) return;
        dot.style.color = 'var(--text-muted)'; dot.title = t('common.status.disconnected');
        setTimeout(() => { if (viewIsCurrent(tok)) connectWs(mount, tok); }, 4000);
    });
    ws.addEventListener('message', (e) => {
        try {
            const m = JSON.parse(e.data);
            if (m.type === 'snapshot' && Array.isArray(m.rows)) {
                rows.clear();
                for (const r of m.rows) rows.set(rowKey(r), r);
            } else if (m.type === 'insider' && m.row) {
                addRow(m.row);
            }
            render();
        } catch (_) {}
    });
}

function rowKey(r) {
    return `${r.symbol}|${r.insider_name}|${r.transaction_date || '-'}|${r.transaction_code}|${r.shares}`;
}

function addRow(r) {
    const k = rowKey(r);
    const fresh = !rows.has(k);
    rows.set(k, r);
    if (fresh && voiceOn && r.kind === 'buy' && r.is_officer && r.dollar_value >= 1_000_000) {
        speak(r);
    }
}

function speak(r) {
    try {
        const mm = (r.dollar_value / 1_000_000).toFixed(1);
        const u = new SpeechSynthesisUtterance(
            `${spell(r.symbol)} insider buy. ${mm} million dollars. ${r.officer_title || 'officer'}.`
        );
        u.rate = 1.1; u.pitch = 1.0; u.volume = 1.0;
        window.speechSynthesis.speak(u);
    } catch (_) {}
}

function spell(s) { return s.split('').join(' '); }

function render() {
    const tbody = document.querySelector('#is-table tbody');
    if (!tbody) return;
    let filtered = Array.from(rows.values()).filter(r => r.dollar_value >= minDollar);
    if (kindFilter !== 'all') {
        filtered = filtered.filter(r => r.kind === kindFilter);
    }
    filtered.sort((a, b) => new Date(b.observed_at) - new Date(a.observed_at));
    if (!filtered.length) {
        tbody.innerHTML = `<tr><td colspan="9" class="muted">${esc(t('view.insider_stream.empty.no_rows'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = filtered.map(r => {
        const cls = r.kind === 'buy' ? 'pos' : r.kind === 'sell' ? 'neg' : 'muted';
        const role = roleStr(r);
        const link = r.filing_link
            ? `<a href="${esc(r.filing_link)}" target="_blank" rel="noopener">${esc(fmtDateTime(r.observed_at))}</a>`
            : esc(fmtDateTime(r.observed_at));
        return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
            <td>${link}</td>
            <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
            <td>${esc(r.insider_name)}</td>
            <td class="muted small">${esc(role)}</td>
            <td class="${cls}"><strong>${esc(r.kind)}</strong> (${esc(r.transaction_code)})</td>
            <td>${fmtShares(r.shares)}</td>
            <td>${r.price_per_share > 0 ? '$' + r.price_per_share.toFixed(2) : '—'}</td>
            <td class="${cls}">${fmtDollar(r.dollar_value)}</td>
            <td>${esc(r.transaction_date || '—')}</td>
        </tr>`;
    }).join('');
}

function roleStr(r) {
    const parts = [];
    if (r.is_officer) parts.push(r.officer_title || 'Officer');
    if (r.is_director) parts.push('Director');
    if (r.is_ten_percent_owner) parts.push('10% Owner');
    return parts.join(', ') || '—';
}

function fmtShares(n) {
    if (n == null) return '—';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(0) + 'K';
    return n.toFixed(0);
}

function fmtDollar(n) {
    if (n == null) return '—';
    const abs = Math.abs(n);
    if (abs >= 1_000_000_000) return '$' + (abs / 1_000_000_000).toFixed(2) + 'B';
    if (abs >= 1_000_000) return '$' + (abs / 1_000_000).toFixed(2) + 'M';
    if (abs >= 1_000) return '$' + (abs / 1_000).toFixed(0) + 'K';
    return '$' + abs.toFixed(0);
}
