// Per-symbol research page — quote + signals + chart + news + fundamentals.
import { api } from '../api.js';
import { ohlcChart } from '../charts.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t, applyUiI18n } from '../i18n.js';

export async function renderResearch(mount, _state, sym) {
    const tok = currentViewToken();
    if (!sym) {
        mount.innerHTML = `
            <h1 data-i18n="view.research.h1.research" class="view-title">// RESEARCH</h1>
            <form id="rs-form" class="inline-form">
                <input name="symbol" placeholder="symbol — AAPL, NVDA, ^GSPC, BTC-USD" data-i18n-placeholder="view.research.placeholder.symbol" required autofocus style="min-width:300px;text-transform:uppercase">
                <button data-i18n="view.research.btn.research" class="primary" type="submit">Research</button>
            </form>
            <p data-i18n="view.research.hint.tip_anything_yahoo_recognizes_works_stocks_indices" class="muted small">Tip: anything Yahoo recognizes works — stocks, indices (^FTSE), futures (CL=F), crypto (BTC-USD).</p>
        `;
        mount.querySelector('#rs-form').addEventListener('submit', (e) => {
            e.preventDefault();
            const s = new FormData(e.target).get('symbol').trim().toUpperCase();
            if (s) window.location.hash = `research/${encodeURIComponent(s)}`;
        });
        return;
    }
    sym = sym.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title">// ${esc(sym)}
            <a class="link small" href="#research">← search another</a>
        </h1>
        <div id="rs-quote" class="cards"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading quote…</div></div></div>
        <div class="chart-panel">
            <h2 data-i18n="view.research.h2.daily_chart_1y">Daily chart (1y)</h2>
            <div id="rs-chart"></div>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.signals_score">Signals + Score</h2>
                <div id="rs-signals">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.indicators">Indicators</h2>
                <div id="rs-indicators">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.pivots_classic">Pivots (classic)</h2>
                <div id="rs-pivots">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.latest_news">Latest News</h2>
                <div id="rs-news">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.analyst_recommendations">Analyst Recommendations</h2>
                <div id="rs-recs">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.fundamentals">Fundamentals</h2>
                <div id="rs-fund">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.earnings">Earnings</h2>
                <div id="rs-earn">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.insider_activity">Insider Activity</h2>
                <div id="rs-ins">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.research.h2.holders">Holders</h2>
                <div id="rs-hold">loading…</div>
            </div>
        </div>
    `;

    // Kick off everything in parallel.
    const q = api.quote(sym).catch(() => null);
    const sig = api.symbolSignals(sym).catch(() => null);
    const news = api.symbolNews(sym, 10).catch(() => []);
    const fund = api.symbolFundamentals(sym).catch(() => null);
    const earn = api.symbolEarnings(sym).catch(() => null);
    const recs = api.symbolRecs(sym).catch(() => null);
    const ins  = api.symbolInsiders(sym).catch(() => null);
    const hold = api.symbolHolders(sym).catch(() => null);

    const to = Math.floor(Date.now() / 1000);
    const from = to - 365 * 86400;
    const bars = api.bars(sym, '1d', from, to).catch(() => ({ bars: [] }));

    const qv = await q;
    if (!viewIsCurrent(tok)) return;
    const quoteEl = mount.querySelector('#rs-quote');
    if (quoteEl) renderQuote(quoteEl, qv);
    bars.then(r => {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#rs-chart');
        if (el) ohlcChart(el, r.bars || [], [], { height: 380 });
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
        ${cell('MACD line', i.macd_line)}${cell('MACD signal', i.macd_signal)}${cell('MACD hist', i.macd_hist)}
        ${cell('RSI(14)', i.rsi14)}${cell('ADX(14)', i.adx14)}
        ${cell('+DI', i.plus_di)}${cell('-DI', i.minus_di)}
        ${cell('Stoch %K', i.stoch_k)}${cell('Stoch %D', i.stoch_d)}
        ${cell('BB upper', i.bb_upper)}${cell('BB middle', i.bb_middle)}${cell('BB lower', i.bb_lower)}
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
        ['Market cap', rawVal(sd.marketCap)],
        ['PE (TTM)',   rawVal(sd.trailingPE)],
        ['Forward PE', rawVal(sd.forwardPE)],
        ['PEG',        rawVal(ks.pegRatio)],
        ['P/B',        rawVal(ks.priceToBook)],
        ['Dividend yield', rawVal(sd.dividendYield)],
        ['52-week high',   rawVal(sd.fiftyTwoWeekHigh)],
        ['52-week low',    rawVal(sd.fiftyTwoWeekLow)],
        ['Beta',           rawVal(sd.beta)],
        ['EPS (TTM)',      rawVal(ks.trailingEps)],
        ['Profit margin',  rawVal(fd.profitMargins)],
        ['Revenue (TTM)',  rawVal(fd.totalRevenue)],
        ['Debt / Equity',  rawVal(fd.debtToEquity)],
        ['ROE',            rawVal(fd.returnOnEquity)],
        ['Sector',         ap.sector || '—'],
        ['Industry',       ap.industry || '—'],
        ['Employees',      rawVal(ap.fullTimeEmployees)],
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
        `).join('')}</tbody></table>` : ''}
    `;
    try { applyUiI18n(el); } catch (_) {}
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
