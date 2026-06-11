// Per-symbol research page — quote + signals + chart + news + fundamentals.
import { api } from '../api.js';
import { createTradingChart } from '../components/trading_chart.js';
import { esc, fmt, fmtDateTime, applyBarWidths } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t, applyUiI18n } from '../i18n.js';

// Normalize user-typed symbols to the form the data layer expects (Yahoo).
//   BTC/USD   → BTC-USD   (slash → dash for any X/Y pair)
//   BTC.USD   → BTC-USD
//   BTC USD   → BTC-USD
//   BTCUSD    → BTC-USD   (common quote-currency suffixes — extend as needed)
//   BTC-USD   → BTC-USD   (passthrough)
// Pure ticker symbols (AAPL, ^GSPC, CL=F) flow through untouched.
const CRYPTO_QUOTE_CURRENCIES = ['USDT', 'USDC', 'USD', 'BUSD', 'EUR', 'GBP', 'JPY', 'BTC', 'ETH'];
function normalizeSymbol(raw) {
    if (!raw) return '';
    let s = String(raw).trim().toUpperCase();
    if (!s) return '';
    // Pair separators: /, \, ., space, colon — collapse to dash.
    if (/[\/\\.: ]/.test(s)) s = s.replace(/[\/\\.: ]+/g, '-');
    // Already has a dash separator: keep as-is (Yahoo's BTC-USD form).
    if (s.includes('-')) return s;
    // Bare concatenated pair like BTCUSD / ETHUSDT — split on the longest
    // quote-currency suffix we recognize. Skip when the symbol contains
    // characters that wouldn't appear in a crypto base (^, =).
    if (/[\^=]/.test(s)) return s;
    for (const qc of CRYPTO_QUOTE_CURRENCIES) {
        if (s.length > qc.length && s.endsWith(qc)) {
            const base = s.slice(0, -qc.length);
            // Heuristic: base must be 2-6 chars of letters/digits to avoid
            // false positives like a ticker that happens to end in "USD".
            if (/^[A-Z0-9]{2,6}$/.test(base)) return `${base}-${qc}`;
        }
    }
    return s;
}

export async function renderResearch(mount, _state, sym) {
    const tok = currentViewToken();
    if (!sym) {
        mount.innerHTML = `
            <h1 data-i18n="view.research.h1.research" class="view-title">// RESEARCH</h1>
            <form id="rs-form" class="inline-form">
                <input name="symbol" placeholder="symbol — AAPL, ^GSPC, CL=F, BTC-USD, BTC/USD" data-i18n-placeholder="view.research.placeholder.symbol"
                       data-tip="view.research.tip.symbol" data-shortcut="focus_search" required autofocus style="min-width:300px;text-transform:uppercase">
                <button data-i18n="view.research.btn.research" data-tip="view.research.tip.submit" data-shortcut="research_action" class="primary" type="submit">Research</button>
            </form>
            <p data-i18n="view.research.hint.tip_anything_yahoo_recognizes_works_stocks_indices" class="muted small">Tip: anything Yahoo recognizes works — stocks, indices (^FTSE), futures (CL=F), crypto (BTC-USD or BTC/USD).</p>
        `;
        mount.querySelector('#rs-form').addEventListener('submit', (e) => {
            e.preventDefault();
            const s = normalizeSymbol(new FormData(e.target).get('symbol'));
            if (s) window.location.hash = `research/${encodeURIComponent(s)}`;
        });
        return;
    }
    const normalized = normalizeSymbol(sym);
    // Bounce the URL to the canonical Yahoo form so reload / share / back-
    // button always land on a working route.
    if (normalized !== String(sym).trim().toUpperCase()) {
        window.location.hash = `research/${encodeURIComponent(normalized)}`;
        return;
    }
    sym = normalized;
    mount.innerHTML = `
        <h1 class="view-title rs-title">
            <span class="rs-title-prefix">//</span>
            <form id="rs-sym-form" class="rs-sym-form" autocomplete="off">
                <input id="rs-sym-input" name="symbol" type="text"
                       value="${esc(sym)}"
                       data-i18n-placeholder="view.research.placeholder.symbol"
                       placeholder="symbol — AAPL, NVDA, ^GSPC, BTC-USD"
                       spellcheck="false" autocapitalize="characters"
                       style="text-transform:uppercase">
                <button type="submit" class="btn btn-secondary rs-sym-go"
                        data-i18n="view.research.btn.research">Research</button>
                <button type="button" id="rs-sym-clear" class="btn btn-secondary rs-sym-clear"
                        data-i18n="view.research.btn.search_another"
                        data-i18n-title="view.research.btn.search_another">← search another</button>
            </form>
        </h1>
        <div id="rs-quote" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="view.research.loading_quote">loading quote…</div></div></div>
        <div class="chart-panel rs-rec-panel">
            <h2 data-i18n="view.research.h2.recommendation">Recommendation</h2>
            <div id="rs-rec"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.research.h2.price_chart">Price chart</h2>
            <div id="rs-chart"></div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.signals_score">Signals + Score</h2>
                <div id="rs-signals"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.indicators">Indicators</h2>
                <div id="rs-indicators"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.pivots_classic">Pivots (classic)</h2>
                <div id="rs-pivots"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.latest_news">Latest News</h2>
                <div id="rs-news"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.analyst_recommendations">Analyst Recommendations</h2>
                <div id="rs-recs"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.fundamentals">Fundamentals</h2>
                <div id="rs-fund"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.earnings">Earnings</h2>
                <div id="rs-earn"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.insider_activity">Insider Activity</h2>
                <div id="rs-ins"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.holders">Holders</h2>
                <div id="rs-hold"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.profile">Company Profile</h2>
                <div id="rs-profile"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.peers">Peers / Sympathy</h2>
                <div id="rs-peers"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.upgrades">Analyst Upgrades / Downgrades</h2>
                <div id="rs-upgrades"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.fundamental_health">Fundamental Health (Piotroski · Altman · Graham)</h2>
                <div id="rs-health"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.gap_stats">Gap Statistics</h2>
                <div id="rs-gaps"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.seasonality">Monthly Seasonality</h2>
                <div id="rs-seasonality"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
        </div>
    `;

    // Inline symbol form — submit to navigate, clear to wipe + focus.
    const symForm  = mount.querySelector('#rs-sym-form');
    const symInput = mount.querySelector('#rs-sym-input');
    const symClear = mount.querySelector('#rs-sym-clear');
    if (symForm && symInput) {
        symForm.addEventListener('submit', (e) => {
            e.preventDefault();
            const next = normalizeSymbol(symInput.value);
            if (next && next !== sym) {
                window.location.hash = `research/${encodeURIComponent(next)}`;
            } else if (!next) {
                symInput.focus();
            }
        });
    }
    if (symClear && symInput) {
        symClear.addEventListener('click', (e) => {
            e.preventDefault();
            symInput.value = '';
            symInput.focus();
        });
    }

    // Kick off everything in parallel.
    const q = api.quote(sym).catch(() => null);
    const rec = api.symbolRecommendation(sym).catch(() => null);
    const recBt = api.symbolRecommendationBacktest(sym).catch(() => null);
    const recWatchers = api.recommendationWatchers().catch(() => []);
    const recWebhooks = api.webhooks?.().catch(() => []);
    const sig = api.symbolSignals(sym).catch(() => null);
    const news = api.symbolNews(sym, 10).catch(() => []);
    const fund = api.symbolFundamentals(sym).catch(() => null);
    const earn = api.symbolEarnings(sym).catch(() => null);
    const recs = api.symbolRecs(sym).catch(() => null);
    const ins  = api.symbolInsiders(sym).catch(() => null);
    const hold = api.symbolHolders(sym).catch(() => null);
    const prof = api.symbolProfile(sym).catch(() => null);
    const peer = api.symbolPeers(sym).catch(() => null);
    const upgr = api.symbolUpgrades(sym).catch(() => null);
    const health = api.symbolFundamentalHealth(sym).catch(() => null);
    const r40 = api.symbolRuleOf40(sym).catch(() => null);
    const beneish = api.symbolBeneish(sym).catch(() => null);
    const chowder = api.symbolChowder(sym).catch(() => null);
    const deepValue = api.symbolDeepValue(sym).catch(() => null);
    const gaps = api.symbolGapStats(sym).catch(() => null);
    const seas = api.symbolSeasonality(sym).catch(() => null);

    const qv = await q;
    if (!viewIsCurrent(tok)) return;
    const quoteEl = mount.querySelector('#rs-quote');
    if (quoteEl) renderQuote(quoteEl, qv);
    const chartEl = mount.querySelector('#rs-chart');
    if (chartEl) createTradingChart(chartEl, { symbol: sym, interval: '1d', height: 380 });
    Promise.all([rec, recBt, recWatchers, recWebhooks]).then(([r, bt, watchers, webhooks]) => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-rec');
        if (el) renderRecommendation(el, r, bt, watchers || [], webhooks || [], sym);
    });
    sig.then(s => {
        if (!viewIsCurrent(tok)) return;
        renderSignals(s, mount);
    });
    news.then(n => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-news');
        if (el) renderNews(el, n);
    });
    fund.then(f => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-fund');
        if (el) renderFund(el, f);
    });
    earn.then(e => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-earn');
        if (el) renderEarnings(el, e);
    });
    recs.then(r => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-recs');
        if (el) renderRecs(el, r);
    });
    ins.then(i => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-ins');
        if (el) renderInsiders(el, i);
    });
    hold.then(h => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-hold');
        if (el) renderHolders(el, h);
    });
    prof.then(p => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-profile');
        if (el) renderProfile(el, p);
    });
    peer.then(p => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-peers');
        if (el) renderPeers(el, p);
    });
    upgr.then(u => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-upgrades');
        if (el) renderUpgrades(el, u);
    });
    Promise.all([health, r40, beneish, chowder, deepValue]).then(([h, r, b, c, dv]) => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-health');
        if (el) renderFundamentalHealth(el, h, r, b, c, dv);
    });
    gaps.then(g => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-gaps');
        if (el) renderGapStats(el, g);
    });
    seas.then(sn => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-seasonality');
        if (el) renderSeasonality(el, sn);
    });
}

function renderFundamentalHealth(el, h, r40 = null, beneish = null, chowder = null, deepValue = null) {
    if (!h) { el.innerHTML = `<p class="muted">${esc(t('common.empty.no_data'))}</p>`; return; }
    const r40Card = r40 ? `
        <div class="card"><div class="label">Rule of 40</div>
            <div class="value ${r40.passes ? 'pos' : 'neg'}">${r40.score.toFixed(1)}</div>
            <div class="small muted">${r40.revenue_growth_pct.toFixed(1)}% growth + ${r40.fcf_margin_pct.toFixed(1)}% FCF margin</div>
        </div>` : '';
    const beneishCard = beneish ? `
        <div class="card"><div class="label">Beneish M-Score</div>
            <div class="value ${beneish.likely_manipulator ? 'neg' : 'pos'}">${beneish.m_score.toFixed(2)}</div>
            <div class="small ${beneish.likely_manipulator ? 'neg' : 'muted'}">${beneish.likely_manipulator ? 'manipulation flag (> −1.78)' : 'clean (≤ −1.78)'}${beneish.missing.length ? ' · ' + beneish.missing.length + ' inputs approximated' : ''}</div>
        </div>` : '';
    const chowderCard = chowder ? `
        <div class="card"><div class="label">Chowder Number</div>
            <div class="value ${chowder.passes ? 'pos' : 'neg'}">${chowder.chowder_number.toFixed(1)}</div>
            <div class="small muted">${chowder.dividend_yield_pct.toFixed(1)}% yield + ${chowder.dividend_cagr_5y_pct.toFixed(1)}% 5y div CAGR vs ${chowder.threshold}</div>
        </div>` : '';
    const ncavCard = deepValue && deepValue.ncav_per_share != null ? `
        <div class="card"><div class="label">Graham NCAV</div>
            <div class="value ${deepValue.is_net_net ? 'pos' : 'neutral'}">$${deepValue.ncav_per_share.toFixed(2)}</div>
            <div class="small ${deepValue.is_net_net ? 'pos' : 'muted'}">${deepValue.is_net_net ? 'NET-NET — under ⅔ NCAV' : deepValue.price_to_ncav != null ? (deepValue.price_to_ncav).toFixed(2) + '× NCAV' : 'negative NCAV'}</div>
        </div>` : '';
    const amCard = deepValue && deepValue.acquirers_multiple != null ? `
        <div class="card"><div class="label">Acquirer's Multiple</div>
            <div class="value ${deepValue.acquirers_multiple < 8 ? 'pos' : deepValue.acquirers_multiple > 16 ? 'neg' : 'neutral'}">${deepValue.acquirers_multiple.toFixed(1)}×</div>
            <div class="small muted">EV / operating earnings (FY${deepValue.year})</div>
        </div>` : '';
    const syCard = deepValue && deepValue.shareholder_yield_pct != null ? `
        <div class="card"><div class="label">Shareholder Yield</div>
            <div class="value ${deepValue.shareholder_yield_pct >= 0 ? 'pos' : 'neg'}">${deepValue.shareholder_yield_pct.toFixed(1)}%</div>
            <div class="small muted">div ${deepValue.dividend_yield_pct != null ? deepValue.dividend_yield_pct.toFixed(1) : '—'}% · buyback ${deepValue.net_buyback_yield_pct != null ? deepValue.net_buyback_yield_pct.toFixed(1) : '—'}% · debt ${deepValue.net_debt_paydown_yield_pct != null ? deepValue.net_debt_paydown_yield_pct.toFixed(1) : '—'}%</div>
        </div>` : '';
    const fCls = h.piotroski_score >= 7 ? 'pos' : h.piotroski_score <= 3 ? 'neg' : 'neutral';
    const zoneCls = { safe: 'pos', grey: 'neutral', distress: 'neg' }[h.altman_zone] || '';
    const checks = (h.piotroski_checks || []).map(c => `
        <div class="rs-health-check">
            <span class="rs-health-mark ${c.passed === true ? 'pos' : c.passed === false ? 'neg' : 'muted'}">
                ${c.passed === true ? '✔' : c.passed === false ? '✘' : '—'}
            </span>
            <span class="rs-health-label">${esc(c.label)}</span>
            <span class="rs-health-detail muted small">${esc(c.detail)}</span>
        </div>`).join('');
    const grahamRow = h.graham_number != null ? `
        <div class="card"><div class="label">Graham Number</div>
            <div class="value">$${Number(h.graham_number).toFixed(2)}</div>
            ${h.graham_upside_pct != null ? `<div class="small ${h.graham_upside_pct >= 0 ? 'pos' : 'neg'}">${(h.graham_upside_pct >= 0 ? '+' : '') + h.graham_upside_pct.toFixed(1)}% vs price</div>` : ''}
        </div>` : '';
    el.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Piotroski F-Score</div>
                <div class="value ${fCls}">${h.piotroski_score} / ${h.piotroski_available}</div></div>
            ${h.altman_z != null ? `
            <div class="card"><div class="label">Altman Z (approx)</div>
                <div class="value ${zoneCls}">${Number(h.altman_z).toFixed(2)}</div>
                <div class="small muted">${esc(h.altman_zone || '')}</div></div>` : ''}
            ${grahamRow}
            ${r40Card}
            ${beneishCard}
            ${chowderCard}
            ${ncavCard}
            ${amCard}
            ${syCard}
        </div>
        <div class="rs-health-checks">${checks}</div>
    `;
}

function renderGapStats(el, g) {
    if (!g) { el.innerHTML = `<p class="muted">${esc(t('common.empty.no_data'))}</p>`; return; }
    const sideRow = (label, s) => `
        <tr><td>${label}</td><td>${s.count}</td>
            <td>${s.avg_gap_pct.toFixed(2)}%</td>
            <td>${s.same_day_fill_rate_pct.toFixed(0)}%</td>
            <td>${s.window_fill_rate_pct.toFixed(0)}%</td>
            <td>${s.avg_sessions_to_fill.toFixed(1)}</td></tr>`;
    el.innerHTML = `
        <table class="gs-table">
            <thead><tr><th>Side</th><th>Count</th><th>Avg gap</th>
                <th>Same-day fill</th><th>5-day fill</th><th>Avg fill (sess)</th></tr></thead>
            <tbody>
                ${sideRow('Gap up', g.up)}
                ${sideRow('Gap down', g.down)}
            </tbody>
        </table>
        <p class="muted small">Threshold ±${g.threshold_pct}% · ${g.bars_analyzed} bars analyzed.</p>
    `;
}

function renderSeasonality(el, sn) {
    if (!sn || !sn.by_month || !sn.by_month.length) {
        el.innerHTML = `<p class="muted">${esc(t('common.empty.no_data'))}</p>`;
        return;
    }
    const NAMES = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
    const cells = sn.by_month.map(m => {
        // mean_return / hit_rate arrive as fractions (0.012 = +1.2%).
        const avg = Number(m.mean_return) * 100;
        const hit = Number(m.hit_rate) * 100;
        const cls = avg > 0.5 ? 'pos' : avg < -0.5 ? 'neg' : 'neutral';
        return `
            <div class="rs-seas-cell ${cls}" title="${m.sample_count} samples, σ ${(Number(m.std_return) * 100).toFixed(1)}%">
                <div class="rs-seas-month">${NAMES[(m.month || 1) - 1]}</div>
                <div class="rs-seas-val">${(avg >= 0 ? '+' : '') + avg.toFixed(1)}%</div>
                <div class="rs-seas-wr muted small">${hit.toFixed(0)}% up</div>
            </div>`;
    }).join('');
    el.innerHTML = `
        <div class="rs-seas-grid">${cells}</div>
        <p class="muted small">${sn.n_observations} monthly observations.</p>
    `;
}

function renderProfile(el, p) {
    if (!p || typeof p !== 'object' || !Object.keys(p).length) {
        el.innerHTML = `<p class="muted" data-i18n="view.research.empty.no_profile">No profile data.</p>`;
        return;
    }
    const rows = [
        [t('view.research.profile.name'),     p.name],
        [t('view.research.profile.ticker'),   p.ticker],
        [t('view.research.profile.exchange'), p.exchange],
        [t('view.research.profile.country'),  p.country],
        [t('view.research.profile.currency'), p.currency],
        [t('view.research.profile.industry'), p.finnhubIndustry],
        [t('view.research.profile.ipo'),      p.ipo],
        [t('view.research.profile.market_cap'),
            p.marketCapitalization != null ? '$' + (p.marketCapitalization).toLocaleString() + 'M' : null],
        [t('view.research.profile.share_outstanding'),
            p.shareOutstanding != null ? p.shareOutstanding.toLocaleString() + 'M' : null],
        [t('view.research.profile.weburl'),
            p.weburl ? `<a href="${esc(p.weburl)}" target="_blank" rel="noopener">${esc(p.weburl)}</a>` : null, true],
        [t('view.research.profile.phone'),    p.phone],
    ];
    el.innerHTML = `<table class="trades"><tbody>${rows
        .filter(([_, v]) => v != null && v !== '')
        .map(([k, v, html]) => `<tr><td>${k}</td><td>${html ? v : esc(String(v))}</td></tr>`)
        .join('')}</tbody></table>`;
    if (p.logo) {
        el.insertAdjacentHTML('afterbegin',
            `<img src="${esc(p.logo)}" alt="" style="max-height:48px;margin-bottom:8px;display:block">`);
    }
}

function renderPeers(el, p) {
    const peers = Array.isArray(p) ? p : [];
    if (!peers.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.research.empty.no_peers">No peer data.</p>`;
        return;
    }
    el.innerHTML = peers.map(sym =>
        `<a class="tile-badge link" style="margin:3px;display:inline-block" href="#research/${esc(sym)}">${esc(sym)}</a>`
    ).join('');
}

function renderUpgrades(el, u) {
    const rows = Array.isArray(u) ? u : [];
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.research.empty.no_upgrades">No analyst actions.</p>`;
        return;
    }
    // Finnhub returns objects like {company, fromGrade, toGrade, action,
    // gradeTime}. Sort newest-first defensively.
    const sorted = [...rows].sort((a, b) => (b.gradeTime || 0) - (a.gradeTime || 0));
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.research.upgrades.date">Date</th>
            <th data-i18n="view.research.upgrades.firm">Firm</th>
            <th data-i18n="view.research.upgrades.action">Action</th>
            <th data-i18n="view.research.upgrades.from">From</th>
            <th data-i18n="view.research.upgrades.to">To</th>
        </tr></thead>
        <tbody>${sorted.slice(0, 30).map(r => {
            const cls = /upgrade|init|reiterate/i.test(r.action) ? 'pos'
                      : /downgrade|cut|sell/i.test(r.action) ? 'neg' : '';
            const date = r.gradeTime
                ? new Date(r.gradeTime * 1000).toLocaleDateString()
                : '—';
            return `<tr>
                <td class="muted">${esc(date)}</td>
                <td>${esc(r.company || '—')}</td>
                <td class="${cls}">${esc(r.action || '—')}</td>
                <td class="muted">${esc(r.fromGrade || '—')}</td>
                <td>${esc(r.toGrade || '—')}</td>
            </tr>`;
        }).join('')}</tbody>
    </table>`;
}

// stockinvest.us-style featured panel. Verdict badge + 5-star rating
// + composite 0-100 score + 30-day target with upside percentage +
// per-indicator breakdown bars. All numbers come from
// /api/symbols/:sym/recommendation backed by stock_recommendation.rs.
function renderRecommendation(el, r, backtest = null, watchers = [], webhooks = [], symbol = '') {
    if (!r) {
        el.innerHTML = `<div class="boot">${esc(t('view.research.empty.no_recommendation'))}</div>`;
        return;
    }
    const VERDICT_LABEL = {
        strong_buy: 'STRONG BUY',
        buy: 'BUY',
        hold: 'HOLD',
        sell: 'SELL',
        strong_sell: 'STRONG SELL',
    };
    const VERDICT_CLS = {
        strong_buy: 'pos strong',
        buy: 'pos',
        hold: 'neutral',
        sell: 'neg',
        strong_sell: 'neg strong',
    };
    const label = VERDICT_LABEL[r.verdict] || r.verdict.toUpperCase();
    const vcls = VERDICT_CLS[r.verdict] || '';
    const stars = '★'.repeat(r.stars) + '☆'.repeat(5 - r.stars);
    const score = Math.round(r.score);
    const target = r.target_price != null ? Number(r.target_price).toFixed(2) : '—';
    const upside = r.upside_pct;
    const upsideCls = upside > 0 ? 'pos' : upside < 0 ? 'neg' : '';
    const upsideStr = upside != null ? (upside >= 0 ? '+' : '') + upside.toFixed(1) + '%' : '—';
    const compBars = (r.components || []).map(c => {
        const sc = Math.round(c.score);
        const barCls = sc >= 60 ? 'pos' : sc <= 40 ? 'neg' : 'neutral';
        return `
            <div class="rs-rec-comp">
                <div class="rs-rec-comp-head">
                    <span class="rs-rec-comp-label">${esc(c.label)}</span>
                    <span class="rs-rec-comp-score ${barCls}">${sc}</span>
                </div>
                <div class="rs-rec-bar"><div class="rs-rec-bar-fill ${barCls}" data-bar-pct="${sc}"></div></div>
                <div class="rs-rec-comp-note muted small">${esc(c.note || '')}</div>
            </div>
        `;
    }).join('');
    const RISK_LABEL = { low: 'LOW RISK', medium: 'MED RISK', high: 'HIGH RISK' };
    const RISK_CLS = { low: 'pos', medium: 'neutral', high: 'neg' };
    const riskBadge = r.risk_level
        ? `<span class="rs-rec-risk ${RISK_CLS[r.risk_level] || ''}"
                 title="annualized vol ${Number(r.annualized_vol_pct).toFixed(1)}%">
               ${RISK_LABEL[r.risk_level] || r.risk_level.toUpperCase()}
           </span>`
        : '';
    const srRow = (r.support != null || r.resistance != null) ? `
        <div class="rs-rec-sr muted small">
            ${r.support != null ? `Support <strong>$${Number(r.support).toFixed(2)}</strong>` : ''}
            ${r.support != null && r.resistance != null ? ' · ' : ''}
            ${r.resistance != null ? `Resistance <strong>$${Number(r.resistance).toFixed(2)}</strong>` : ''}
        </div>` : '';
    const fcRow = (r.forecast_3m_low != null && r.forecast_3m_high != null) ? `
        <div class="rs-rec-fc muted small">
            3-month range <strong>$${Number(r.forecast_3m_low).toFixed(2)}</strong>
            – <strong>$${Number(r.forecast_3m_high).toFixed(2)}</strong>
        </div>` : '';
    el.innerHTML = `
        <div class="rs-rec-head">
            <div class="rs-rec-verdict-wrap">
                <div class="rs-rec-verdict ${vcls}">${label}</div>
                ${riskBadge}
            </div>
            <div class="rs-rec-stars" aria-label="${r.stars} of 5 stars">${stars}</div>
            <div class="rs-rec-score-block">
                <div class="rs-rec-score">${score}</div>
                <div class="rs-rec-score-label muted small">SCORE / 100</div>
            </div>
            <div class="rs-rec-target-block">
                <div class="rs-rec-target">$${target}</div>
                <div class="muted small">${r.horizon_days}-DAY TARGET</div>
                <div class="rs-rec-upside ${upsideCls}">${upsideStr}</div>
            </div>
        </div>
        ${srRow}${fcRow}
        <div class="rs-rec-breakdown">${compBars}</div>
        ${renderBacktestSummary(backtest)}
        ${renderWatcherWidget(symbol, watchers, webhooks)}
        <div class="muted small rs-rec-foot">Composite of ${r.components.length} components, ${r.bars_analyzed} bars analyzed.</div>
    `;
    try { applyBarWidths(el); } catch (_) {}
    wireWatcherWidget(el, symbol, watchers, webhooks);
}

function renderBacktestSummary(bt) {
    if (!bt || !bt.by_verdict || !bt.by_verdict.length) return '';
    const rows = bt.by_verdict.map(s => {
        const cls = (s.verdict === 'strong_buy' || s.verdict === 'buy') ? 'pos'
                  : (s.verdict === 'strong_sell' || s.verdict === 'sell') ? 'neg' : 'neutral';
        return `
            <tr>
                <td><span class="gs-verdict ${cls}">${esc((s.verdict || '').toUpperCase().replace('_', ' '))}</span></td>
                <td>${s.sample_count}</td>
                <td>${s.hit_rate_pct.toFixed(1)}%</td>
                <td class="${s.avg_forward_return_pct >= 0 ? 'pos' : 'neg'}">${(s.avg_forward_return_pct >= 0 ? '+' : '') + s.avg_forward_return_pct.toFixed(2)}%</td>
            </tr>
        `;
    }).join('');
    return `
        <div class="rs-rec-bt">
            <div class="rs-rec-bt-title muted small">Historical accuracy (last ${bt.bars_used} bars, ${bt.horizon_bars}-bar horizon)</div>
            <table class="rs-rec-bt-table">
                <thead><tr><th>Verdict</th><th>Samples</th><th>Hit rate</th><th>Avg return</th></tr></thead>
                <tbody>${rows}</tbody>
            </table>
        </div>
    `;
}

function renderWatcherWidget(symbol, watchers, webhooks) {
    if (!symbol) return '';
    const existing = (watchers || []).find(w => w.symbol === symbol);
    const whOpts = (webhooks || []).filter(w => w.enabled).map(w =>
        `<label class="rs-rec-wh"><input type="checkbox" data-wh="${esc(w.id)}" ${existing && existing.webhook_ids && existing.webhook_ids.includes(w.id) ? 'checked' : ''}> ${esc(w.label || w.kind || w.url)}</label>`
    ).join('');
    if (!whOpts) {
        return `<div class="rs-rec-watcher muted small">No webhooks configured — add one in Settings to enable verdict alerts for ${esc(symbol)}.</div>`;
    }
    return `
        <div class="rs-rec-watcher">
            <div class="rs-rec-watcher-head">
                <strong>Watch verdict changes for ${esc(symbol)}</strong>
                ${existing ? `<span class="rs-rec-watcher-status">last verdict: ${esc(existing.last_verdict || '—')}</span>` : ''}
            </div>
            <div class="rs-rec-watcher-row">${whOpts}</div>
            <div class="rs-rec-watcher-actions">
                <button class="btn btn-secondary rs-rec-watch-save">${existing ? 'Update' : 'Watch'}</button>
                ${existing ? `<button class="btn btn-secondary rs-rec-watch-del" data-id="${esc(existing.id)}">Remove</button>` : ''}
            </div>
        </div>
    `;
}

function wireWatcherWidget(el, symbol, watchers, webhooks) {
    if (!el || !symbol) return;
    const saveBtn = el.querySelector('.rs-rec-watch-save');
    const delBtn = el.querySelector('.rs-rec-watch-del');
    if (saveBtn) {
        saveBtn.addEventListener('click', async () => {
            const ids = Array.from(el.querySelectorAll('input[data-wh]:checked'))
                .map(cb => cb.getAttribute('data-wh'));
            saveBtn.disabled = true;
            try {
                await api.recommendationWatcherUpsert({
                    symbol,
                    webhook_ids: ids,
                    fire_on: null,
                    enabled: true,
                });
                saveBtn.textContent = 'Saved';
            } catch (e) {
                saveBtn.textContent = 'Failed';
            } finally {
                setTimeout(() => { saveBtn.disabled = false; saveBtn.textContent = 'Update'; }, 2500);
            }
        });
    }
    if (delBtn) {
        delBtn.addEventListener('click', async () => {
            const id = delBtn.getAttribute('data-id');
            if (!id) return;
            delBtn.disabled = true;
            try {
                await api.recommendationWatcherDelete(id);
                delBtn.textContent = 'Removed';
            } catch (e) {
                delBtn.textContent = 'Failed';
            } finally {
                setTimeout(() => { delBtn.disabled = false; delBtn.textContent = 'Remove'; }, 2500);
            }
        });
    }
}

function renderQuote(el, q) {
    if (!q) { el.innerHTML = `<div class="boot">${esc(t('view.research.empty.no_quote'))}</div>`; return; }
    const ch = q.change_pct;
    const cls = ch >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="card"><div class="label" data-i18n="view.research.card.price">Price</div><div class="value">${fmt(q.price)}</div></div>
        <div class="card"><div class="label" data-i18n="view.research.card.change">Change</div>
            <div class="value ${cls}">${ch != null ? (ch >= 0 ? '+' : '') + ch.toFixed(2) + '%' : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.research.card.prev_close">Prev close</div><div class="value">${q.prev_close != null ? fmt(q.prev_close) : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.research.card.day_hi_lo">Day Hi/Lo</div>
            <div class="value">${q.day_high != null ? fmt(q.day_high) : '—'} / ${q.day_low != null ? fmt(q.day_low) : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.research.card.volume">Volume</div><div class="value">${q.volume != null ? q.volume.toLocaleString() : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.research.card.market">Market</div><div class="value">${q.market_state || '—'}</div></div>
    `;
    try { applyUiI18n(el); } catch (_) {}
}

function renderSignals(s, mount) {
    const sigEl = mount.querySelector('#rs-signals');
    const indEl = mount.querySelector('#rs-indicators');
    const pivEl = mount.querySelector('#rs-pivots');
    if (!sigEl || !indEl || !pivEl) return;
    if (!s) { sigEl.textContent = t('common.empty.no_data'); indEl.textContent = ''; pivEl.textContent = ''; return; }
    const cls = s.score >= 3 ? 'pos' : s.score <= -3 ? 'neg' : '';
    sigEl.innerHTML = `
        <div class="score-card ${cls}">
            <div class="score-num">${s.score >= 0 ? '+' : ''}${s.score}</div>
            <div class="score-label">${s.summary.toUpperCase()}</div>
        </div>
        <table class="trades" style="margin-top:8px">
            <thead><tr><th data-i18n="view.research.th.signal">Signal</th><th data-i18n="view.research.th.side">Side</th><th data-i18n="view.research.th.weight">Weight</th><th data-i18n="view.research.th.detail">Detail</th></tr></thead>
            <tbody>${s.signals.map(sig => `
                <tr>
                    <td>${esc(sig.name)}</td>
                    <td class="${sig.side === 'buy' ? 'pos' : 'neg'}">${sig.side}</td>
                    <td>${sig.weight >= 0 ? '+' : ''}${sig.weight}</td>
                    <td class="muted">${esc(sig.detail)}</td>
                </tr>`).join('') || `<tr><td colspan="4" class="muted">${esc(t('view.research.empty.signals'))}</td></tr>`}
            </tbody>
        </table>
    `;
    const i = s.indicators;
    const cell = (label, v) => `<tr><td>${label}</td><td>${v != null ? fmt(Number(v)) : '—'}</td></tr>`;
    indEl.innerHTML = `<table class="trades"><tbody>
        ${cell('SMA(20)', i.sma20)}${cell('SMA(50)', i.sma50)}${cell('SMA(200)', i.sma200)}
        ${cell('EMA(12)', i.ema12)}${cell('EMA(26)', i.ema26)}
        ${cell(t('view.research.ind.macd_line'), i.macd_line)}${cell(t('view.research.ind.macd_signal'), i.macd_signal)}${cell(t('view.research.ind.macd_hist'), i.macd_hist)}
        ${cell('RSI(14)', i.rsi14)}${cell('ADX(14)', i.adx14)}
        ${cell('+DI', i.plus_di)}${cell('-DI', i.minus_di)}
        ${cell(t('view.research.ind.stoch_k'), i.stoch_k)}${cell(t('view.research.ind.stoch_d'), i.stoch_d)}
        ${cell(t('view.research.ind.bb_upper'), i.bb_upper)}${cell(t('view.research.ind.bb_middle'), i.bb_middle)}${cell(t('view.research.ind.bb_lower'), i.bb_lower)}
    </tbody></table>`;
    if (s.pivots) {
        const p = s.pivots;
        pivEl.innerHTML = `<table class="trades"><tbody>
            <tr><td>R3</td><td class="neg">${fmt(p.r3)}</td></tr>
            <tr><td>R2</td><td class="neg">${fmt(p.r2)}</td></tr>
            <tr><td>R1</td><td class="neg">${fmt(p.r1)}</td></tr>
            <tr><td><strong data-i18n="view.research.pivot.pivot">Pivot</strong></td><td><strong>${fmt(p.pivot)}</strong></td></tr>
            <tr><td>S1</td><td class="pos">${fmt(p.s1)}</td></tr>
            <tr><td>S2</td><td class="pos">${fmt(p.s2)}</td></tr>
            <tr><td>S3</td><td class="pos">${fmt(p.s3)}</td></tr>
        </tbody></table>`;
    } else {
        pivEl.innerHTML = '<p data-i18n="view.research.hint.need_at_least_2_daily_bars" class="muted">Need at least 2 daily bars.</p>';
    }
    try { applyUiI18n(pivEl); } catch (_) {}
}

function renderNews(el, items) {
    if (!items || !items.length) { el.innerHTML = '<p data-i18n="view.research.hint.no_news" class="muted">No news.</p>'; return; }
    el.innerHTML = items.map(n => `
        <div class="news-item">
            <a href="${esc(n.link || '#')}" target="_blank" rel="noopener noreferrer">${esc(n.title || '(no title)')}</a>
            <div class="meta">${esc(n.publisher || '')} ${n.provider_publish_time
                ? '· ' + new Date(n.provider_publish_time * 1000).toLocaleString(undefined, { hour12: false })
                : ''}</div>
        </div>`).join('');
}

function renderRecs(el, r) {
    if (!r) { el.innerHTML = '<p data-i18n="view.research.hint.no_data" class="muted">no data</p>'; return; }
    const trend = r.recommendationTrend?.trend || [];
    if (!trend.length) { el.innerHTML = '<p data-i18n="view.research.hint.no_analyst_data" class="muted">No analyst data.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th data-i18n="view.research.th.period">Period</th><th data-i18n="view.research.th.strong_buy">Strong Buy</th><th data-i18n="view.research.th.buy">Buy</th><th data-i18n="view.research.th.hold">Hold</th><th data-i18n="view.research.th.sell">Sell</th><th data-i18n="view.research.th.strong_sell">Strong Sell</th></tr></thead>
        <tbody>${trend.map(t => `
            <tr><td>${esc(t.period)}</td>
                <td class="pos">${t.strongBuy ?? '—'}</td>
                <td class="pos">${t.buy ?? '—'}</td>
                <td>${t.hold ?? '—'}</td>
                <td class="neg">${t.sell ?? '—'}</td>
                <td class="neg">${t.strongSell ?? '—'}</td></tr>
        `).join('')}</tbody></table>`;
}

function rawVal(v) {
    if (v == null) return '—';
    if (typeof v === 'object' && 'raw' in v) return v.raw;
    if (typeof v === 'object' && 'fmt' in v) return v.fmt;
    return v;
}

function renderFund(el, f) {
    if (!f) { el.innerHTML = '<p data-i18n="view.research.hint.no_data_2" class="muted">no data</p>'; return; }
    const sd = f.summaryDetail || {};
    const ks = f.defaultKeyStatistics || {};
    const fd = f.financialData || {};
    const ap = f.assetProfile || {};
    const rows = [
        [t('view.research.fundamentals.market_cap'),     rawVal(sd.marketCap)],
        [t('view.research.fundamentals.pe_ttm'),         rawVal(sd.trailingPE)],
        [t('view.research.fundamentals.forward_pe'),     rawVal(sd.forwardPE)],
        ['PEG',                                          rawVal(ks.pegRatio)],
        ['P/B',                                          rawVal(ks.priceToBook)],
        [t('view.research.fundamentals.dividend_yield'), rawVal(sd.dividendYield)],
        [t('view.research.fundamentals.52w_high'),       rawVal(sd.fiftyTwoWeekHigh)],
        [t('view.research.fundamentals.52w_low'),        rawVal(sd.fiftyTwoWeekLow)],
        [t('view.research.fundamentals.beta'),           rawVal(sd.beta)],
        [t('view.research.fundamentals.eps_ttm'),        rawVal(ks.trailingEps)],
        [t('view.research.fundamentals.profit_margin'),  rawVal(fd.profitMargins)],
        [t('view.research.fundamentals.revenue_ttm'),    rawVal(fd.totalRevenue)],
        [t('view.research.fundamentals.debt_equity'),    rawVal(fd.debtToEquity)],
        ['ROE',                                          rawVal(fd.returnOnEquity)],
        [t('view.research.fundamentals.sector'),         ap.sector || '—'],
        [t('view.research.fundamentals.industry'),       ap.industry || '—'],
        [t('view.research.fundamentals.employees'),      rawVal(ap.fullTimeEmployees)],
    ];
    el.innerHTML = `<table class="trades"><tbody>${rows.map(([k, v]) =>
        `<tr><td>${k}</td><td>${esc(String(v))}</td></tr>`).join('')}</tbody></table>`;
    if (ap.longBusinessSummary) {
        el.insertAdjacentHTML('beforeend',
            `<details style="margin-top:8px"><summary data-i18n="view.research.summary.business">Business summary</summary>
             <p class="muted small" style="margin-top:6px">${esc(ap.longBusinessSummary)}</p>
             </details>`);
    }
}

function renderEarnings(el, e) {
    if (!e) { el.innerHTML = '<p data-i18n="view.research.hint.no_data_3" class="muted">no data</p>'; return; }
    const cal = e.calendarEvents?.earnings || {};
    const next = cal.earningsDate?.[0]?.fmt;
    const eps = cal.earningsAverage?.fmt;
    const rev = cal.revenueAverage?.fmt;
    const hist = e.earningsHistory?.history || [];
    el.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.research.card.next_earnings">Next earnings</div><div class="value">${next || '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.research.card.eps_est">EPS est.</div><div class="value">${eps || '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.research.card.revenue_est">Revenue est.</div><div class="value">${rev || '—'}</div></div>
        </div>
        ${hist.length ? `<table class="trades"><thead><tr>
            <th data-i18n="view.research.th.period_2">Period</th><th data-i18n="view.research.th.eps_est">EPS Est</th><th data-i18n="view.research.th.eps_actual">EPS Actual</th><th data-i18n="view.research.th.surprise">Surprise %</th>
        </tr></thead><tbody>${hist.map(h => `
            <tr><td>${esc(h.period || '')}</td>
                <td>${rawVal(h.epsEstimate)}</td>
                <td>${rawVal(h.epsActual)}</td>
                <td class="${(h.surprisePercent?.raw ?? 0) >= 0 ? 'pos' : 'neg'}">${rawVal(h.surprisePercent)}</td></tr>
        `).join('')}</tbody></table>
        <h3 data-i18n="view.research.h3.surprise_chart">EPS surprise % per period</h3>
        <div id="res-earn-chart" style="width:100%;height:240px"></div>
        <h3 data-i18n="view.research.h3.eps_estvactual_chart">EPS estimate vs actual per period</h3>
        <div id="res-earn-ea-chart" style="width:100%;height:220px"></div>
        <p data-i18n="view.research.hint.eps_estvactual" class="muted small">Estimate vs actual side-by-side. Persistent positive surprises = analyst sandbagging; persistent negatives = optimistic estimates.</p>` : ''}
    `;
    try { applyUiI18n(el); } catch (_) {}
    renderEarningsSurpriseChart(hist);
    renderEarningsEstActChart(hist);
}

function renderEarningsEstActChart(hist) {
    const el = document.getElementById('res-earn-ea-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const pts = (hist || []).filter(h =>
        Number.isFinite(Number(h.epsEstimate?.raw)) && Number.isFinite(Number(h.epsActual?.raw)));
    if (pts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.research.empty_estact_chart">${esc(t('view.research.empty_estact_chart'))}</div>`;
        return;
    }
    const labels = pts.map(h => h.period || '');
    const ests = pts.map(h => Number(h.epsEstimate.raw));
    const acts = pts.map(h => Number(h.epsActual.raw));
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.research.chart.period_idx') },
            { label: t('view.research.chart.eps_estimate'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 12, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: t('view.research.chart.eps_actual'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ests, acts], el);
}

function renderEarningsSurpriseChart(hist) {
    const el = document.getElementById('res-earn-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const pts = (hist || []).filter(h => Number.isFinite(Number(h.surprisePercent?.raw)));
    if (pts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.research.empty_chart">${esc(t('view.research.empty_chart'))}</div>`;
        return;
    }
    const labels = pts.map(h => h.period || '');
    const ys = pts.map(h => Number(h.surprisePercent.raw));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.research.chart.period_idx') },
            { label: t('view.research.chart.surprise'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.research.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderInsiders(el, i) {
    if (!i) { el.innerHTML = '<p data-i18n="view.research.hint.no_data_4" class="muted">no data</p>'; return; }
    const tx = i.insiderTransactions?.transactions || [];
    if (!tx.length) { el.innerHTML = '<p data-i18n="view.research.hint.no_transactions" class="muted">No transactions.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th data-i18n="view.research.th.date">Date</th><th data-i18n="view.research.th.filer">Filer</th><th data-i18n="view.research.th.position">Position</th><th data-i18n="view.research.th.tx">Tx</th><th data-i18n="view.research.th.shares">Shares</th><th data-i18n="view.research.th.value">Value</th></tr></thead>
        <tbody>${tx.slice(0, 15).map(t => `
            <tr><td>${rawVal(t.startDate)}</td>
            <td>${esc(t.filerName || '')}</td>
            <td>${esc(t.filerRelation || '')}</td>
            <td>${esc(t.transactionText || '')}</td>
            <td>${rawVal(t.shares)}</td>
            <td>${rawVal(t.value)}</td></tr>
        `).join('')}</tbody></table>`;
}

function renderHolders(el, h) {
    if (!h) { el.innerHTML = '<p data-i18n="view.research.hint.no_data_5" class="muted">no data</p>'; return; }
    const b = h.majorHoldersBreakdown || {};
    const inst = h.institutionOwnership?.ownershipList || [];
    // When the backend ships an explicit `_source_note` it means every
    // upstream provider was unreachable (Yahoo crumb-locked, Finnhub
    // ownership is premium-only) and the rest of the payload is an empty
    // stub. Render the note as a muted hint above the empty cards so the
    // user knows it's a data-source gap, not a render bug.
    const sourceNote = typeof h._source_note === 'string' ? h._source_note : '';
    const isEmpty = !Object.keys(b).length && !inst.length;
    if (sourceNote && isEmpty) {
        el.innerHTML = `<p class="muted small">${esc(sourceNote)}</p>`;
        return;
    }
    el.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.research.card.insider_pct">Insider %</div><div class="value">${rawVal(b.insidersPercentHeld)}</div></div>
            <div class="card"><div class="label" data-i18n="view.research.card.institutions_pct">Institutions %</div><div class="value">${rawVal(b.institutionsPercentHeld)}</div></div>
            <div class="card"><div class="label" data-i18n="view.research.card.inst_count">Inst. count</div><div class="value">${rawVal(b.institutionsCount)}</div></div>
        </div>
        ${inst.length ? `<table class="trades"><thead><tr><th data-i18n="view.research.th.holder">Holder</th><th data-i18n="view.research.th.out">% out</th><th data-i18n="view.research.th.shares_2">Shares</th><th data-i18n="view.research.th.reported">Reported</th></tr></thead>
        <tbody>${inst.slice(0, 15).map(h => `
            <tr><td>${esc(h.organization || '')}</td>
            <td>${rawVal(h.pctHeld)}</td>
            <td>${rawVal(h.position)}</td>
            <td>${rawVal(h.reportDate)}</td></tr>`).join('')}
        </tbody></table>` : ''}
    `;
    try { applyUiI18n(el); } catch (_) {}
}

// Cell for indicator table — referenced via util to keep dates fresh.
void fmtDateTime;
