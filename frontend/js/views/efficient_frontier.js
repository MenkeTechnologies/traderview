// Efficient frontier (Markowitz) — min-variance + max-Sharpe tangency portfolios
// and the frontier curve, via /calc/efficient-frontier.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
let LAST_DOC = null;

const SEED = [
    { name: 'Stocks', ret: 8, vol: 12 },
    { name: 'Bonds', ret: 12, vol: 18 },
    { name: 'REIT', ret: 15, vol: 25 },
];

function rowHtml(a) {
    return `
        <div class="mpb-row nec-row">
            <input type="text" class="ef-name" placeholder="${esc(t('view.ef.ph.name'))}" value="${esc(a.name || '')}">
            <input type="number" step="0.5" class="ef-ret" value="${a.ret}">
            <input type="number" step="0.5" min="0" class="ef-vol" value="${a.vol}">
            <button type="button" class="ef-del" data-i18n="view.ef.remove">Remove</button>
        </div>`;
}

export async function renderEfficientFrontier(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ef.h1.title">// EFFICIENT FRONTIER</span></h1>
        <p class="muted small" data-i18n="view.ef.hint.intro">
            Traces the Markowitz mean-variance frontier and reports the minimum-variance portfolio and the
            maximum-Sharpe (tangency) portfolio. The covariance matrix is built from each asset's volatility
            and a single constant pairwise correlation. Short-selling is allowed (unconstrained MVO).
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ef.h2.inputs">Assets</h2>
            <form id="ef-form" class="inline-form">
                <label><span data-i18n="view.ef.label.corr">Pairwise correlation (%)</span>
                    <input type="number" step="5" min="-100" max="100" name="correlation_pct" value="30"></label>
                <label><span data-i18n="view.ef.label.rf">Risk-free rate (%)</span>
                    <input type="number" step="0.1" name="risk_free_pct" value="3"></label>
                <label><span data-i18n="view.ef.label.points">Frontier points</span>
                    <input type="number" step="1" min="2" max="100" name="points" value="40"></label>
            </form>
            <div class="mpb-head nec-head">
                <span data-i18n="view.ef.col.name">Asset</span>
                <span data-i18n="view.ef.col.ret">Return (%)</span>
                <span data-i18n="view.ef.col.vol">Volatility (%)</span>
                <span></span>
            </div>
            <div id="ef-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="ef-add" class="secondary" data-i18n="view.ef.add">+ Add asset</button>
        </div>
        <div id="ef-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ef-form');
    const rowsEl = mount.querySelector('#ef-rows');

    const generate = async () => {
        const assets = [...rowsEl.querySelectorAll('.nec-row')].map((r) => ({
            name: (r.querySelector('.ef-name').value || '').trim(),
            expected_return_pct: Number(r.querySelector('.ef-ret').value) || 0,
            volatility_pct: Number(r.querySelector('.ef-vol').value) || 0,
        })).filter((a) => a.name && a.volatility_pct > 0);
        const body = {
            assets,
            correlation_pct: Number(form.querySelector('[name="correlation_pct"]').value) || 0,
            risk_free_pct: Number(form.querySelector('[name="risk_free_pct"]').value) || 0,
            points: Number(form.querySelector('[name="points"]').value) || 40,
        };
        if (assets.length < 2) { mount.querySelector('#ef-result').innerHTML = ''; return; }
        try {
            const doc = await api.calcEfficientFrontier(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ef.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#ef-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ name: '', ret: 10, vol: 15 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('ef-del')) {
            e.target.closest('.nec-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
    generate();
}

function frontierSvg(doc) {
    const pts = doc.frontier;
    if (!pts.length) return '';
    const xs = pts.map((p) => p.volatility_pct);
    const ys = pts.map((p) => p.expected_return_pct);
    const minX = Math.min(...xs, doc.tangency.volatility_pct, doc.min_variance.volatility_pct, 0);
    const maxX = Math.max(...xs, doc.tangency.volatility_pct) * 1.05;
    const minY = Math.min(...ys, 0);
    const maxY = Math.max(...ys) * 1.05;
    const W = 360, H = 240, pad = 30;
    const sx = (v) => pad + (v - minX) / (maxX - minX || 1) * (W - 2 * pad);
    const sy = (v) => H - pad - (v - minY) / (maxY - minY || 1) * (H - 2 * pad);
    const poly = pts.map((p) => `${sx(p.volatility_pct).toFixed(1)},${sy(p.expected_return_pct).toFixed(1)}`).join(' ');
    const dot = (x, y, cls) => `<circle cx="${sx(x).toFixed(1)}" cy="${sy(y).toFixed(1)}" r="4" class="${cls}"></circle>`;
    return `
        <svg viewBox="0 0 ${W} ${H}" class="ef-svg" role="img" aria-label="Efficient frontier">
            <polyline points="${poly}" fill="none" stroke="#00e5ff" stroke-width="1.5"></polyline>
            ${dot(doc.min_variance.volatility_pct, doc.min_variance.expected_return_pct, 'ef-mv')}
            ${dot(doc.tangency.volatility_pct, doc.tangency.expected_return_pct, 'ef-tan')}
            <text x="${pad}" y="${H - 8}" class="ef-axis">vol →</text>
            <text x="6" y="${pad}" class="ef-axis">ret ↑</text>
        </svg>`;
}

function weightsTable(p) {
    return p.weights.map((w) => `<tr><td>${esc(w.name)}</td><td>${pct(w.weight_pct)}</td></tr>`).join('');
}

function docToText(doc) {
    const lines = ['EFFICIENT FRONTIER', ''];
    const fmt = (lbl, p) => {
        lines.push(`${lbl}: return ${p.expected_return_pct}%  vol ${p.volatility_pct}%  sharpe ${p.sharpe}`);
        for (const w of p.weights) lines.push(`  ${w.name}: ${w.weight_pct}%`);
    };
    fmt('Min-variance', doc.min_variance);
    lines.push('');
    fmt('Max-Sharpe (tangency)', doc.tangency);
    lines.push('', `Capital market line slope (Sharpe): ${doc.cml_slope}`);
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#ef-result');
    if (!doc.ok) { el.innerHTML = `<p class="muted" data-i18n="view.ef.singular">No frontier — covariance is singular or inputs invalid.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ef.card.sharpe">Max Sharpe</div>
                    <div class="value">${num(doc.tangency.sharpe)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ef.card.tanret">Tangency return</div>
                    <div class="value">${pct(doc.tangency.expected_return_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ef.card.tanvol">Tangency vol</div>
                    <div class="value">${pct(doc.tangency.volatility_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ef.card.mvvol">Min-var vol</div>
                    <div class="value">${pct(doc.min_variance.volatility_pct)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ef-copy" type="button" data-i18n="view.ef.btn.copy">Copy</button>
            </div>
        </div>
        ${frontierSvg(doc)}
        <div class="ef-tables">
            <table class="data-table"><thead><tr>
                <th data-i18n="view.ef.th.tan">Max-Sharpe weights</th><th></th></tr></thead>
                <tbody>${weightsTable(doc.tangency)}</tbody></table>
            <table class="data-table"><thead><tr>
                <th data-i18n="view.ef.th.mv">Min-variance weights</th><th></th></tr></thead>
                <tbody>${weightsTable(doc.min_variance)}</tbody></table>
        </div>
    `;
    applyUiI18n(el);
    el.querySelector('#ef-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ef.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ef.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
}
