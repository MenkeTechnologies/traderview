// Golden Stars — stockinvest.us-style ranked leaderboard. Pulls the
// most-recent persisted recommendation per symbol and orders by score
// descending. Click a row to drill into the research view.

import { api } from '../api.js';
import { esc, applyBarWidths } from '../util.js';
import { t, applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent, routeIs } from '../app.js';

const FILTER_PRESETS = {
    all: { label: 'All', min_score: 0 },
    buys: { label: 'Buy candidates', min_score: 60 },
    strong: { label: 'Strong buys', min_score: 75 },
};

let timer = null;

export async function renderGoldenStars(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.golden_stars.h1" class="view-title">// GOLDEN STARS</h1>
        <p class="muted small" data-i18n="view.golden_stars.subtitle">
            Top-ranked names by composite score (RSI + MACD + trend + momentum + ADX + volume).
            Refreshes whenever the nightly compute writes new rows; manually re-run via
            POST /api/recommendations/cron/run.
        </p>
        <div class="gs-controls">
            <div class="gs-filter-row">
                ${Object.entries(FILTER_PRESETS).map(([k, v]) => `
                    <button class="btn btn-secondary gs-filter" data-key="${k}">${esc(v.label)}</button>
                `).join('')}
                <span class="muted small gs-meta" id="gs-meta"></span>
                <button class="btn btn-secondary gs-run" id="gs-run" data-i18n="view.golden_stars.btn.run">Run compute now</button>
            </div>
        </div>
        <div class="chart-panel">
            <div id="gs-body">
                <span class="tv-spinner-inline" role="status" aria-label="loading"></span>
            </div>
        </div>
    `;
    try { applyUiI18n(mount); } catch (_) {}

    let activeFilter = 'all';

    const reload = async () => {
        const preset = FILTER_PRESETS[activeFilter];
        const rows = await api.recommendationLeaderboard({
            limit: 100,
            min_score: preset.min_score,
        }).catch(() => []);
        if (!viewIsCurrent(tok)) return;
        renderBody(mount.querySelector('#gs-body'), rows);
        const meta = mount.querySelector('#gs-meta');
        if (meta) {
            const newest = rows.length ? new Date(rows[0].computed_at).toLocaleString() : '—';
            meta.textContent = `${rows.length} symbols · latest ${newest}`;
        }
    };

    mount.querySelectorAll('.gs-filter').forEach(btn => {
        btn.addEventListener('click', () => {
            activeFilter = btn.dataset.key;
            mount.querySelectorAll('.gs-filter').forEach(b => b.classList.toggle('active', b === btn));
            reload();
        });
    });
    mount.querySelector('.gs-filter[data-key="all"]')?.classList.add('active');

    const runBtn = mount.querySelector('#gs-run');
    if (runBtn) {
        runBtn.addEventListener('click', async () => {
            runBtn.disabled = true;
            runBtn.textContent = 'Running…';
            try {
                const res = await api.recommendationCronRun();
                runBtn.textContent = `Done · ${res?.compute?.succeeded || 0} computed, ${res?.fired_alerts || 0} alerts fired`;
            } catch (e) {
                runBtn.textContent = 'Failed';
            } finally {
                setTimeout(() => {
                    runBtn.disabled = false;
                    runBtn.textContent = 'Run compute now';
                }, 4000);
                reload();
            }
        });
    }

    await reload();
    // Poll every 60s so the user sees fresh rows after the cron fires.
    if (timer) clearInterval(timer);
    timer = setInterval(() => { if (viewIsCurrent(tok)) reload(); }, 60_000);
    window.addEventListener('hashchange', () => {
        if (!routeIs('golden-stars')) { clearInterval(timer); timer = null; }
    }, { once: true });
}

function renderBody(el, rows) {
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<div class="boot muted">${esc(t('view.golden_stars.empty') || 'No recommendations yet — click "Run compute now" to seed the leaderboard.')}</div>`;
        return;
    }
    const rowsHtml = rows.map((r, i) => {
        const verdictCls = verdictClass(r.verdict);
        const stars = '★'.repeat(r.stars) + '☆'.repeat(5 - r.stars);
        const upsideCls = r.upside_pct > 0 ? 'pos' : r.upside_pct < 0 ? 'neg' : '';
        const upside = (r.upside_pct >= 0 ? '+' : '') + Number(r.upside_pct).toFixed(1) + '%';
        const scoreInt = Math.round(r.score);
        return `
            <tr class="gs-row" data-symbol="${esc(r.symbol)}">
                <td class="gs-rank">${i + 1}</td>
                <td class="gs-sym"><a href="#research/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                <td><span class="gs-verdict ${verdictCls}">${verdictLabel(r.verdict)}</span></td>
                <td class="gs-stars">${stars}</td>
                <td class="gs-score">
                    <div class="gs-score-num">${scoreInt}</div>
                    <div class="gs-score-bar"><div class="gs-score-bar-fill ${verdictCls}" data-bar-pct="${scoreInt}"></div></div>
                </td>
                <td class="gs-price">$${Number(r.current_price).toFixed(2)}</td>
                <td class="gs-target">$${Number(r.target_price).toFixed(2)}</td>
                <td class="gs-upside ${upsideCls}">${upside}</td>
                <td class="gs-when muted small">${ago(r.computed_at)}</td>
            </tr>
        `;
    }).join('');
    el.innerHTML = `
        <table class="gs-table">
            <thead><tr>
                <th>#</th>
                <th>Symbol</th>
                <th>Verdict</th>
                <th>Stars</th>
                <th>Score</th>
                <th>Price</th>
                <th>Target</th>
                <th>Upside</th>
                <th>When</th>
            </tr></thead>
            <tbody>${rowsHtml}</tbody>
        </table>
    `;
    try { applyBarWidths(el); } catch (_) {}
}

function verdictLabel(v) {
    return ({
        strong_buy: 'STRONG BUY',
        buy: 'BUY',
        hold: 'HOLD',
        sell: 'SELL',
        strong_sell: 'STRONG SELL',
    })[v] || String(v).toUpperCase();
}

function verdictClass(v) {
    return ({
        strong_buy: 'pos strong',
        buy: 'pos',
        hold: 'neutral',
        sell: 'neg',
        strong_sell: 'neg strong',
    })[v] || '';
}

function ago(iso) {
    if (!iso) return '—';
    const ms = Date.now() - new Date(iso).getTime();
    if (ms < 60_000) return 'just now';
    if (ms < 3_600_000) return `${Math.floor(ms / 60_000)}m ago`;
    if (ms < 86_400_000) return `${Math.floor(ms / 3_600_000)}h ago`;
    return `${Math.floor(ms / 86_400_000)}d ago`;
}
