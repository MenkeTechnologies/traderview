// USPTO Patents — Finnhub /stock/uspto-patent.
// Per-symbol patent filings + recent grants. R&D-intensive companies'
// patent velocity signals tech momentum (semis, biotech, EVs).

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderUsptoPatents(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.uspto.h1.title">// USPTO PATENTS</span></h1>
        <p class="muted small" data-i18n="view.uspto.hint.intro">
            US Patent &amp; Trademark Office filings by symbol. R&amp;D-intensive companies'
            patent velocity correlates with tech momentum (semis, biotech, EVs).
            Surge in patents = pre-launch R&amp;D signal.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="up-form">
                <label><span data-i18n="view.uspto.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="NVDA" required></label>
                <button class="primary" type="submit" data-i18n="view.uspto.btn.load">Load</button>
            </form>
            <div id="up-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('up-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('up-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const to = new Date();
        const from = new Date(to);
        from.setFullYear(from.getFullYear() - 5);
        const data = await api.symbolUsptoPatent(state.symbol, fmtDay(from), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.uspto.empty">No patents on record.</p>`;
            return;
        }
        // Yearly patent count for velocity analysis.
        const yearly = new Map();
        for (const r of rows) {
            const y = (r.publicationDate || r.filingDate || '').slice(0, 4);
            if (!y) continue;
            yearly.set(y, (yearly.get(y) || 0) + 1);
        }
        const years = [...yearly.keys()].sort();
        const sorted = [...rows].sort((a, b) =>
            String(b.publicationDate || b.filingDate || '').localeCompare(
                String(a.publicationDate || a.filingDate || '')));
        el.innerHTML = `
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.uspto.card.total">Total patents (5y)</div>
                    <div class="value">${rows.length}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.uspto.card.yearly_avg">Yearly average</div>
                    <div class="value">${(rows.length / Math.max(1, years.length)).toFixed(1)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.uspto.card.peak_year">Peak year</div>
                    <div class="value">${[...yearly.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] || '—'}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.uspto.card.latest_year">${years[years.length - 1] || '—'}</div>
                    <div class="value">${yearly.get(years[years.length - 1]) || 0}</div>
                </div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.uspto.th.publication_date">Publication</th>
                    <th data-i18n="view.uspto.th.filing_date">Filed</th>
                    <th data-i18n="view.uspto.th.title">Title</th>
                    <th data-i18n="view.uspto.th.number">Patent #</th>
                    <th data-i18n="view.uspto.th.url">Link</th>
                </tr></thead>
                <tbody>${sorted.slice(0, 200).map(r => `
                    <tr>
                        <td>${esc(r.publicationDate || '—')}</td>
                        <td class="muted">${esc(r.filingDate || '—')}</td>
                        <td>${esc(r.title || '—')}</td>
                        <td class="muted small">${esc(r.patentNumber || r.applicationNumber || '—')}</td>
                        <td>${r.url ? `<a class="link" href="${esc(r.url)}" target="_blank" rel="noopener">view</a>` : '—'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.uspto.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.uspto.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
