// PEAD (post-earnings-announcement drift) — ranks recent earnings
// surprises by how the stock drifted in the days after the
// announcement. Positive score = drift in same direction as surprise.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

let lookbackDays = 90;
let viewMode = 'recent'; // 'recent' | 'top'

export async function renderPead(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pead.title">// POST-EARNINGS DRIFT (PEAD)</span></h1>
        <p class="muted small" data-i18n-html="view.pead.intro">
            Ball &amp; Brown 1968 → Chordia/Shivakumar 2006: stocks beating consensus EPS
            drift positively for ~60 trading days; misses drift negatively. Score =
            sign(surprise) × return_20d. Positive score means the drift confirmed the
            surprise direction — a tradeable PEAD setup. Filter is min |surprise| ≥ 2%.
        </p>
        <div class="chart-panel">
            <div class="pead-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <div class="pead-toggle" role="tablist" aria-label="view-mode">
                    <button class="btn btn-sm" data-mode="recent" data-i18n="view.pead.btn.recent">Recent</button>
                    <button class="btn btn-sm" data-mode="top"    data-i18n="view.pead.btn.top">Top Drift</button>
                </div>
                <label>
                    <span data-i18n="view.pead.label.lookback">lookback (days)</span>
                    <input type="number" id="pead-days" min="14" max="365" step="7" value="${lookbackDays}" style="width:80px">
                </label>
                <button class="btn btn-sm" id="pead-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
            </div>
            <table class="trades" id="pead-table">
                <thead><tr>
                    <th data-i18n="view.pead.th.earnings_date">Earnings Date</th>
                    <th data-i18n="view.pead.th.symbol">Symbol</th>
                    <th data-i18n="view.pead.th.surprise">Surprise %</th>
                    <th data-i18n="view.pead.th.day0">Day 0 %</th>
                    <th data-i18n="view.pead.th.r5">+5d %</th>
                    <th data-i18n="view.pead.th.r20">+20d %</th>
                    <th data-i18n="view.pead.th.r60">+60d %</th>
                    <th data-i18n="view.pead.th.score">Score</th>
                </tr></thead>
                <tbody><tr><td colspan="8" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelectorAll('[data-mode]').forEach(b => {
        b.addEventListener('click', () => { viewMode = b.dataset.mode; applyToggleState(mount); fetchAndRender(mount); });
    });
    mount.querySelector('#pead-days').addEventListener('change', (e) => {
        const v = parseInt(e.target.value, 10);
        if (Number.isFinite(v) && v >= 14) { lookbackDays = v; fetchAndRender(mount); }
    });
    mount.querySelector('#pead-refresh').addEventListener('click', () => fetchAndRender(mount));
    applyToggleState(mount);
    fetchAndRender(mount);
}

function applyToggleState(mount) {
    mount.querySelectorAll('[data-mode]').forEach(b => {
        b.classList.toggle('active', b.dataset.mode === viewMode);
    });
}

async function fetchAndRender(mount) {
    const tbody = mount.querySelector('#pead-table tbody');
    if (!tbody) return;
    tbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('common.loading'))}</td></tr>`;
    const path = viewMode === 'top' ? '/pead/top-drift' : '/pead/recent';
    try {
        const rows = await api(`${path}?days=${lookbackDays}&limit=100`);
        if (!rows || !rows.length) {
            tbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('view.pead.empty.no_rows'))}</td></tr>`;
            return;
        }
        tbody.innerHTML = rows.map(r => {
            const scoreCls = r.score == null ? 'muted'
                          : r.score >= 0 ? 'pos' : 'neg';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td>${esc(r.earnings_date)}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td class="${r.surprise_pct >= 0 ? 'pos' : 'neg'}">${fmtPct(r.surprise_pct)}</td>
                <td class="${r.return_day0_pct == null ? 'muted' : r.return_day0_pct >= 0 ? 'pos' : 'neg'}">${fmtPct(r.return_day0_pct)}</td>
                <td class="${r.return_5d_pct == null ? 'muted' : r.return_5d_pct >= 0 ? 'pos' : 'neg'}">${fmtPct(r.return_5d_pct)}</td>
                <td class="${r.return_20d_pct == null ? 'muted' : r.return_20d_pct >= 0 ? 'pos' : 'neg'}">${fmtPct(r.return_20d_pct)}</td>
                <td class="${r.return_60d_pct == null ? 'muted' : r.return_60d_pct >= 0 ? 'pos' : 'neg'}">${fmtPct(r.return_60d_pct)}</td>
                <td class="${scoreCls}"><strong>${fmtPct(r.score)}</strong></td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(String(e))}</td></tr>`;
    }
}

function fmtPct(n) {
    if (n == null) return '—';
    return (n >= 0 ? '+' : '') + n.toFixed(2) + '%';
}
