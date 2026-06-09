// Index Constituents — pick an index (^GSPC = S&P 500, ^NDX = Nasdaq-100,
// ^DJI = Dow 30, etc.) and list its current members. Useful for
// sector-rotation and sympathy-play screening.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const INDICES = [
    { value: '^GSPC',  label: 'S&P 500' },
    { value: '^NDX',   label: 'Nasdaq 100' },
    { value: '^DJI',   label: 'Dow 30' },
    { value: '^RUT',   label: 'Russell 2000' },
    { value: '^FTSE',  label: 'FTSE 100' },
    { value: '^N225',  label: 'Nikkei 225' },
    { value: '^HSI',   label: 'Hang Seng' },
    { value: '^GDAXI', label: 'DAX' },
    { value: '^FCHI',  label: 'CAC 40' },
    { value: '^STOXX50E', label: 'Euro Stoxx 50' },
];

let state = { index: '^GSPC' };

export async function renderIndexConstituents(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.index_const.h1.title">// INDEX CONSTITUENTS</span></h1>
        <p class="muted small" data-i18n="view.index_const.hint.intro">
            Current members of the selected index. Click a symbol to research it.
            S&amp;P 500 + Nasdaq 100 lists are free; international indices may require premium.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.index_const.label.index">Index</span>
                    <select id="idx-sel">${INDICES.map(i =>
                        `<option value="${i.value}" ${i.value === state.index ? 'selected' : ''}>${esc(i.label)} (${esc(i.value)})</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="idx-refresh" type="button" data-i18n="view.index_const.btn.refresh">Refresh</button>
            </div>
            <div id="idx-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('idx-sel').addEventListener('change', e => {
        state.index = e.target.value;
        void load(tok);
    });
    document.getElementById('idx-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const el = document.getElementById('idx-result');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.finnhubIndexConstituents(state.index);
        if (!viewIsCurrent(tok)) return;
        const consts = data?.constituents || [];
        if (!consts.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.index_const.empty">No constituents (may require premium for this index).</p>`;
            return;
        }
        const breakdown = data?.constituentsBreakdown || [];
        const byTicker = new Map(breakdown.map(b => [b.symbol, b]));
        const sorted = [...consts].sort((a, b) =>
            (byTicker.get(b)?.weight || 0) - (byTicker.get(a)?.weight || 0));
        el.innerHTML = `
            <p class="muted small">
                <span data-i18n="view.index_const.label.count">Constituents:</span>
                <strong>${consts.length}</strong>
            </p>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.index_const.th.rank">#</th>
                    <th data-i18n="view.index_const.th.symbol">Symbol</th>
                    <th data-i18n="view.index_const.th.name">Name</th>
                    <th data-i18n="view.index_const.th.weight">Weight %</th>
                </tr></thead>
                <tbody>${sorted.slice(0, 500).map((sym, i) => {
                    const b = byTicker.get(sym) || {};
                    return `<tr>
                        <td class="muted">${i + 1}</td>
                        <td><a class="link" href="#research/${esc(sym)}">${esc(sym)}</a></td>
                        <td class="muted">${esc(b.name || '—')}</td>
                        <td>${typeof b.weight === 'number' ? b.weight.toFixed(2) + '%' : '—'}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.index_const.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.index_const.toast.failed'), { level: 'error' });
    }
}
