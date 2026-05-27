// Market breadth — NYSE TICK / TRIN / A-D / Up-Down Vol / Put-Call + regime.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let timer = null;

export async function renderBreadth(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// MARKET BREADTH</h1>
        <p class="muted small">Intraday tape regime: NYSE TICK (instantaneous up-tick count),
            TRIN (Arms Index — volume bias), Advance-Decline issues, Up-Down volume,
            CBOE Put-Call ratio. Composite score combines all five into a -100..+100
            regime gauge. Polls every 60s.</p>

        <div id="bcomp" class="cards"><div class="boot">loading…</div></div>
        <div id="binds"></div>
        <div class="chart-panel">
            <h2>Regime guide</h2>
            <table class="trades">
                <thead><tr><th>Indicator</th><th>Strong bull</th><th>Mild bull</th><th>Neutral</th><th>Mild bear</th><th>Strong bear</th></tr></thead>
                <tbody>
                    <tr><td>NYSE TICK</td><td class="pos">≥ +800</td><td class="pos">+400..+800</td><td>±400</td><td class="neg">−400..−800</td><td class="neg">≤ −800</td></tr>
                    <tr><td>NYSE TRIN</td><td class="pos">≤ 0.5</td><td class="pos">0.5..0.9</td><td>0.9..1.1</td><td class="neg">1.1..2.0</td><td class="neg">≥ 2.0</td></tr>
                    <tr><td>Advance−Decline</td><td class="pos">≥ +1500</td><td class="pos">+500..+1500</td><td>±500</td><td class="neg">−500..−1500</td><td class="neg">≤ −1500</td></tr>
                    <tr><td>Put-Call ratio</td><td class="neg">≤ 0.6 *</td><td class="pos">0.6..0.8</td><td>0.8..1.0</td><td class="neg">1.0..1.2</td><td class="pos">≥ 1.2 *</td></tr>
                </tbody>
            </table>
            <p class="muted small">* Put-Call is a contrarian indicator at extremes — very low PCR = complacency (often near tops), very high PCR = fear (often near bottoms).</p>
        </div>
    `;
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok);
    }, 60_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#breadth')) { clearInterval(timer); timer = null; }
    }, { once: true });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    try {
        const s = await api.breadthSnapshot();
        if (!viewIsCurrent(tok)) return;
        renderComposite(s, mount);
        renderIndicators(s, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#bcomp');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderComposite(s, mount) {
    const regCls = s.regime === 'bullish' ? 'pos' : s.regime === 'bearish' ? 'neg' : '';
    const scoreCls = s.composite_score >= 30 ? 'pos' : s.composite_score <= -30 ? 'neg' : '';
    const el = mount.querySelector('#bcomp');
    if (!el) return;
    el.innerHTML = `
        <div class="card"><div class="label">Composite score</div>
            <div class="value ${scoreCls}">${s.composite_score >= 0 ? '+' : ''}${s.composite_score}</div></div>
        <div class="card"><div class="label">Regime</div>
            <div class="value ${regCls}">${s.regime.toUpperCase()}</div></div>
        <div class="card"><div class="label">Indicators fired</div>
            <div class="value">${[s.tick, s.trin, s.addn, s.vold, s.pcr].filter(Boolean).length} / 5</div></div>
        <div class="card"><div class="label">Updated</div>
            <div class="value small">${new Date(s.fetched_at).toLocaleTimeString(undefined, { hour12: false })}</div></div>
    `;
}

function renderIndicators(s, mount) {
    const inds = [s.tick, s.trin, s.addn, s.vold, s.pcr].filter(Boolean);
    const el = mount.querySelector('#binds');
    if (!el) return;
    if (!inds.length) {
        el.innerHTML = '<p class="boot">No breadth tickers returned data — try in market hours.</p>';
        return;
    }
    el.innerHTML = `
        <div class="cards">
            ${inds.map(i => {
                const chCls = i.change_pct >= 0 ? 'pos' : 'neg';
                return `<div class="card">
                    <div class="label">${esc(i.label)} (${esc(i.symbol)})</div>
                    <div class="value">${fmt(i.value, Math.abs(i.value) < 10 ? 3 : 0)}</div>
                    <div class="small ${chCls}">${i.change_pct >= 0 ? '+' : ''}${i.change_pct.toFixed(2)}%</div>
                    <div class="muted small">${esc(i.interpretation)}</div>
                </div>`;
            }).join('')}
        </div>
    `;
}
