// Pre-market futures dashboard — index futures, commodities, crypto, FX.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let timer = null;

export async function renderPremarket(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// PRE-MARKET / OVERNIGHT</h1>
        <p class="muted small">Cross-asset overnight tape: index futures, commodities, crypto, FX.
            Each gap is normalized by 20-day ATR — magnitudes above 1.0× ATR are statistically
            significant moves vs the security's own recent volatility. High-importance economic
            releases scheduled for today appear at the bottom. Refreshes every 30s.</p>

        <div id="pmEvents"></div>
        <div id="pmContent" class="cards"><div class="boot">loading…</div></div>
    `;
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
        if (!viewIsCurrent(tok)) { clearInterval(timer); timer = null; return; }
        refresh(mount, tok);
    }, 30_000);
    window.addEventListener('hashchange', () => {
        if (!window.location.hash.startsWith('#premarket')) { clearInterval(timer); timer = null; }
    }, { once: true });
    await refresh(mount, tok);
}

async function refresh(mount, tok) {
    try {
        const s = await api.premarketSnapshot();
        if (!viewIsCurrent(tok)) return;
        renderGroups(s.contracts, mount);
        renderEvents(s.today_events, s.fetched_at, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#pmContent');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderGroups(contracts, mount) {
    const groups = new Map();
    for (const c of contracts) {
        if (!groups.has(c.group)) groups.set(c.group, []);
        groups.get(c.group).push(c);
    }
    const out = [];
    for (const [grp, rows] of groups) {
        out.push(`<h2 class="view-title" style="margin-top:1rem;">// ${esc(grp).toUpperCase()}</h2>`);
        out.push(`<div class="cards">${rows.map(card).join('')}</div>`);
    }
    const el = mount.querySelector('#pmContent');
    if (el) el.innerHTML = out.join('');
}

function card(c) {
    if (!c.price) {
        return `<div class="card"><div class="label">${esc(c.symbol)}</div>
            <div class="muted small">no data</div></div>`;
    }
    const ch = c.change_pct;
    const chCls = ch == null ? '' : (ch >= 0 ? 'pos' : 'neg');
    const chTxt = ch == null ? '—' : `${ch >= 0 ? '+' : ''}${ch.toFixed(2)}%`;
    const atrTxt = c.atr_pct == null ? '—' : `${c.atr_pct.toFixed(2)}%`;
    let magTxt = '—', magCls = '';
    if (c.atr_multiple != null) {
        magTxt = `${c.atr_multiple.toFixed(2)}× ATR`;
        if (c.atr_multiple >= 1.5) magCls = 'neg';
        else if (c.atr_multiple >= 1.0) magCls = 'warn';
    }
    const rng = (c.day_high != null && c.day_low != null)
        ? `<div class="muted small">range ${fmt(c.day_low)} – ${fmt(c.day_high)}</div>`
        : '';
    const ms = c.market_state ? `<div class="muted small">${esc(c.market_state.toLowerCase())}</div>` : '';
    return `<div class="card">
        <div class="label">${esc(c.label)} (${esc(c.symbol)})</div>
        <div class="value">${fmt(c.price, c.price < 10 ? 4 : 2)}</div>
        <div class="small ${chCls}">${chTxt}</div>
        <div class="muted small">ATR(20) ${atrTxt}</div>
        <div class="small ${magCls}">${magTxt}</div>
        ${rng}${ms}
    </div>`;
}

function renderEvents(events, fetched, mount) {
    const el = mount.querySelector('#pmEvents');
    if (!el) return;
    if (!events || !events.length) {
        el.innerHTML = `
            <div class="chart-panel">
                <h2>Today's high-impact releases</h2>
                <p class="muted small">No high-importance scheduled releases today (per static
                    economic calendar). Updated ${new Date(fetched).toLocaleTimeString(undefined, { hour12: false })}.</p>
            </div>`;
        return;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2>Today's high-impact releases (${events.length})</h2>
            <table class="trades">
                <thead><tr><th>Time (ET)</th><th>Event</th><th>Category</th><th>Source</th></tr></thead>
                <tbody>
                    ${events.map(e => `<tr>
                        <td>${esc(e.when_et.split('T')[1].slice(0, 5))}</td>
                        <td>${esc(e.name)}</td>
                        <td>${esc(e.category)}</td>
                        <td class="small muted">${esc(e.source)}</td>
                    </tr>`).join('')}
                </tbody>
            </table>
            <p class="muted small">Updated ${new Date(fetched).toLocaleTimeString(undefined, { hour12: false })}</p>
        </div>
    `;
}
