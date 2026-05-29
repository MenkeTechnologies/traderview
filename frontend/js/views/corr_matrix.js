// Correlation matrix view — pairwise Pearson heatmap + leaderboards.
import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderCorrMatrix(mount) {
    const tok = currentViewToken();
    const wls = await api.watchlists().catch(() => []);
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.corr_matrix.h1.correlation_matrix" class="view-title">// CORRELATION MATRIX</h1>
        <p data-i18n="view.corr_matrix.hint.pairwise_pearson_correlation_on_log_returns_of_cac" class="muted small">Pairwise Pearson correlation on log-returns of cached daily bars
            with intersected dates (pairs need ≥30 common observations to score). Red = strongly
            positive (moves together — overlap risk if both long), green = strongly negative
            (true diversifier), grey = uncorrelated. The diagonal is fixed at 1.</p>

        <div class="chart-panel">
            <form id="cm-form" class="inline-form">
                <label><span data-i18n="view.corr_matrix.label.source">Source</span>
                    <select name="mode">
                        <option data-i18n="view.corr_matrix.opt.watchlist" value="watchlist">watchlist</option>
                        <option data-i18n="view.corr_matrix.opt.custom_symbols" value="symbols">custom symbols</option>
                    </select>
                </label>
                <label id="cm-wl-label"><span data-i18n="view.corr_matrix.label.watchlist">Watchlist</span>
                    <select name="watchlist_id">
                        ${wls.map(w => `<option value="${w.id}">${esc(w.name)}${w.is_default ? ' ★' : ''}</option>`).join('')}
                    </select>
                </label>
                <label id="cm-syms-label" style="display:none;"><span data-i18n="view.corr_matrix.label.symbols">Symbols</span>
                    <input name="symbols" placeholder="SPY,QQQ,IWM,DIA,GLD,TLT,USO,XLE"
                           data-i18n-placeholder="view.corr_matrix.placeholder.symbols" style="min-width:280px;">
                </label>
                <label><span data-i18n="view.corr_matrix.label.days">Days</span>
                    <input name="days" type="number" min="30" max="730" value="90" style="width:80px;">
                </label>
                <button data-i18n="view.corr_matrix.btn.compute" class="primary" type="submit">Compute</button>
                <span id="cm-status" class="muted small"></span>
            </form>
        </div>

        <div id="cm-out"><p data-i18n="view.corr_matrix.hint.pick_a_watchlist_or_paste_symbols_and_compute" class="muted small">Pick a watchlist or paste symbols and compute.</p></div>
    `;
    const modeSel = mount.querySelector('#cm-form [name=mode]');
    modeSel.addEventListener('change', () => {
        const isSyms = modeSel.value === 'symbols';
        const wlLabel = mount.querySelector('#cm-wl-label');
        const symLabel = mount.querySelector('#cm-syms-label');
        if (wlLabel) wlLabel.style.display = isSyms ? 'none' : '';
        if (symLabel) symLabel.style.display = isSyms ? '' : 'none';
    });
    mount.querySelector('#cm-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const status = mount.querySelector('#cm-status');
        const out = mount.querySelector('#cm-out');
        if (!out) return;
        out.innerHTML = '<div class="boot">computing… (one bar fetch per symbol)</div>';
        if (status) status.textContent = '';
        try {
            const days = Number(fd.get('days')) || 90;
            const r = fd.get('mode') === 'symbols'
                ? await api.corrSymbols(fd.get('symbols').trim().toUpperCase(), days)
                : await api.corrWatchlist(fd.get('watchlist_id'), days);
            if (!viewIsCurrent(tok)) return;
            const out2 = mount.querySelector('#cm-out');
            if (out2) render(r, out2);
            const status2 = mount.querySelector('#cm-status');
            if (status2) status2.textContent = `${r.symbols.length} symbols · ${r.pairs.length} pairs`;
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const out2 = mount.querySelector('#cm-out');
            if (out2) out2.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function colorForCorr(v) {
    if (v == null) return '#1a1d2e';
    const t = Math.max(-1, Math.min(1, v));
    // -1 → green, 0 → near-black, +1 → red. Diagonal (1.0) auto-red.
    if (t >= 0) {
        // 0..+1 → grey..red
        const g = Math.round(40 + 50 * (1 - t));
        const r = Math.round(80 + 175 * t);
        return `rgb(${r},${g},${g})`;
    } else {
        const r = Math.round(40 + 50 * (1 - Math.abs(t)));
        const g = Math.round(80 + 175 * Math.abs(t));
        return `rgb(${r},${g},${r})`;
    }
}

function render(r, out) {
    out.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.corr_matrix.h2.heatmap">Heatmap</h2>
            <div style="overflow:auto;">
                <table class="corr-matrix">
                    <thead>
                        <tr><th></th>${r.symbols.map(s => `<th>${esc(s)}</th>`).join('')}</tr>
                    </thead>
                    <tbody>
                        ${r.symbols.map((s, i) => `<tr>
                            <th>${esc(s)}</th>
                            ${r.values[i].map((v, j) => {
                                const bg = colorForCorr(v);
                                const txt = v == null ? '·' : v.toFixed(2);
                                const border = i === j ? 'border:2px solid #00ffaa;' : '';
                                return `<td style="background:${bg};color:#000;${border}">${txt}</td>`;
                            }).join('')}
                        </tr>`).join('')}
                    </tbody>
                </table>
            </div>
            <p class="muted small">${r.days}-day window · computed ${new Date(r.computed_at).toLocaleString()}</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.corr_matrix.h2.most_correlated_pairs_overlap_risk_if_both_long">Most correlated pairs (overlap risk if both long)</h2>
            ${pairTable(r.top_correlated)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.corr_matrix.h2.most_diversifying_pairs_best_hedges">Most diversifying pairs (best hedges)</h2>
            ${pairTable(r.top_diversifying)}
        </div>
    `;
}

function pairTable(pairs) {
    if (!pairs.length) return '<p data-i18n="view.corr_matrix.hint.no_pairs" class="muted small">no pairs</p>';
    return `<table class="trades">
        <thead><tr><th data-i18n="view.corr_matrix.th.pair">Pair</th><th>ρ</th><th data-i18n="view.corr_matrix.th.samples">Samples</th></tr></thead>
        <tbody>
        ${pairs.map(p => {
            const v = p.value;
            const cls = v >= 0.7 ? 'neg' : v <= -0.3 ? 'pos' : 'muted';
            return `<tr>
                <td>${esc(p.a)} ↔ ${esc(p.b)}</td>
                <td class="${cls}">${v == null ? '—' : v.toFixed(3)}</td>
                <td class="small muted">${p.samples}</td>
            </tr>`;
        }).join('')}
        </tbody>
    </table>`;
}
