// SEC Filings Browser — Finnhub /stock/filings.
// Per-symbol 10-K / 10-Q / 8-K / 13F filings list with form-type filter.
// Quick-access alternative to going to EDGAR; same data, faster UX.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const FORMS = [
    { value: '',     label: 'All forms' },
    { value: '10-K', label: '10-K (annual)' },
    { value: '10-Q', label: '10-Q (quarterly)' },
    { value: '8-K',  label: '8-K (material event)' },
    { value: '13F-HR', label: '13F (institutional holdings)' },
    { value: '4',    label: '4 (insider transaction)' },
    { value: 'S-1',  label: 'S-1 (registration)' },
    { value: 'S-3',  label: 'S-3 (registration shelf)' },
    { value: 'DEF 14A', label: 'DEF 14A (proxy)' },
];

let state = { symbol: '', form: '' };

export async function renderFilingsBrowser(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.filings.h1.title">// SEC FILINGS BROWSER</span></h1>
        <p class="muted small" data-i18n="view.filings.hint.intro">
            Per-symbol SEC filings. 8-K = material event (M&amp;A, earnings, exec changes —
            highest catalyst weight). 4 = insider transactions. 13F = institutional changes.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="fil-form">
                <label><span data-i18n="view.filings.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <label><span data-i18n="view.filings.label.form">Form</span>
                    <select name="form">${FORMS.map(f =>
                        `<option value="${f.value}" ${f.value === state.form ? 'selected' : ''}>${esc(f.label)}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" type="submit" data-i18n="view.filings.btn.load">Load</button>
            </form>
            <div id="fil-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fil-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.form = fd.get('form') || '';
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('fil-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const to = new Date();
        const from = new Date(to);
        from.setFullYear(from.getFullYear() - 2);
        const data = await api.symbolFilings(state.symbol, fmtDay(from), fmtDay(to), state.form || undefined);
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(data) ? data : (data?.filings || data?.data || []);
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.filings.empty">No filings.</p>`;
            return;
        }
        const sorted = [...rows].sort((a, b) =>
            String(b.filedDate || b.acceptedDate || '').localeCompare(String(a.filedDate || a.acceptedDate || '')));
        // Group counts by form-type.
        const counts = new Map();
        for (const r of sorted) {
            const k = r.form || r.formType || '—';
            counts.set(k, (counts.get(k) || 0) + 1);
        }
        const countsHtml = [...counts.entries()]
            .sort((a, b) => b[1] - a[1])
            .map(([k, n]) =>
                `<span class="tile-badge" style="margin:2px;display:inline-block">${esc(k)} <strong>${n}</strong></span>`)
            .join('');
        el.innerHTML = `
            <p class="muted small">${countsHtml}</p>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.filings.th.filed">Filed</th>
                    <th data-i18n="view.filings.th.accepted">Accepted</th>
                    <th data-i18n="view.filings.th.form">Form</th>
                    <th data-i18n="view.filings.th.access_number">Access #</th>
                    <th data-i18n="view.filings.th.report_url">Report</th>
                    <th data-i18n="view.filings.th.filing_url">Filing</th>
                </tr></thead>
                <tbody>${sorted.slice(0, 200).map(r => `
                    <tr>
                        <td><strong>${esc(r.filedDate || '—')}</strong></td>
                        <td class="muted">${esc(r.acceptedDate || '—')}</td>
                        <td>${esc(r.form || r.formType || '—')}</td>
                        <td class="muted small">${esc(r.accessNumber || '—')}</td>
                        <td>${r.reportUrl ? `<a class="link" href="${esc(r.reportUrl)}" target="_blank" rel="noopener">view</a>` : '—'}</td>
                        <td>${r.filingUrl ? `<a class="link" href="${esc(r.filingUrl)}" target="_blank" rel="noopener">view</a>` : '—'}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.filings.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.filings.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
