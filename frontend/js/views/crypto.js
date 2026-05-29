// Crypto pack — CoinGecko top-N + global stats + BTC on-chain dashboard.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n } from '../i18n.js';

const compact = (n) => {
    if (n == null) return '—';
    const abs = Math.abs(n);
    if (abs >= 1e12) return '$' + (n / 1e12).toFixed(2) + 'T';
    if (abs >= 1e9)  return '$' + (n / 1e9).toFixed(2)  + 'B';
    if (abs >= 1e6)  return '$' + (n / 1e6).toFixed(2)  + 'M';
    if (abs >= 1e3)  return '$' + (n / 1e3).toFixed(2)  + 'K';
    return '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });
};
const num = (n) => n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => n == null ? '—' : (n >= 0 ? '+' : '') + Number(n).toFixed(2) + '%';

export async function renderCrypto(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.crypto.h1.crypto" class="view-title">// CRYPTO</h1>
        <div id="c-glob" class="cards">loading…</div>
        <div class="chart-panel">
            <h2 data-i18n="view.crypto.h2.top_100_by_market_cap">Top-100 by market cap</h2>
            <div id="c-table">loading…</div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.crypto.h2.bitcoin_on_chain">Bitcoin on-chain</h2>
            <div id="c-onchain" class="cards">loading…</div>
        </div>
    `;
    try {
        const [g, top, chain] = await Promise.all([
            api.cryptoGlobal().catch(() => null),
            api.cryptoMarkets(100).catch(() => []),
            api.cryptoBtcChain().catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        if (g) renderGlobal(g, mount);
        renderTable(top, mount);
        if (chain) renderChain(chain, mount);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const el = mount.querySelector('#c-table');
        if (el) el.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderGlobal(g, mount) {
    const el = mount.querySelector('#c-glob');
    if (!el) return;
    el.innerHTML = `
        <div class="card"><div class="label" data-i18n="view.crypto.card.total_mcap">Total mcap</div><div class="value">${compact(g.total_market_cap_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.volume_24h">24h volume</div><div class="value">${compact(g.total_volume_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.mcap_delta_24h">Mcap Δ 24h</div>
            <div class="value ${g.market_cap_change_24h_pct >= 0 ? 'pos' : 'neg'}">${pct(g.market_cap_change_24h_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.btc_dominance">BTC dominance</div><div class="value">${g.btc_dominance.toFixed(2)}%</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.eth_dominance">ETH dominance</div><div class="value">${g.eth_dominance.toFixed(2)}%</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.active_coins">Active coins</div><div class="value">${num(g.active_cryptocurrencies)}</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.markets">Markets</div><div class="value">${num(g.markets)}</div></div>
    `;
    try { applyUiI18n(el); } catch (_) {}
}

function renderTable(rows, mount) {
    const el = mount.querySelector('#c-table');
    if (!el) return;
    if (!rows.length) { el.innerHTML = '<p data-i18n="view.crypto.hint.coingecko_rate_limit_hit_retry_in_a_minute" class="muted">CoinGecko rate limit hit — retry in a minute.</p>'; return; }
    el.innerHTML = `<table class="trades">
        <thead><tr><th>#</th><th></th><th data-i18n="view.crypto.th.coin">Coin</th><th data-i18n="view.crypto.th.price">Price</th>
            <th data-i18n="view.crypto.th.24h">24h</th><th data-i18n="view.crypto.th.7d">7d</th><th data-i18n="view.crypto.th.mcap">Mcap</th><th data-i18n="view.crypto.th.vol_24h">Vol 24h</th>
            <th data-i18n="view.crypto.th.circ_supply">Circ supply</th><th data-i18n="view.crypto.th.ath">ATH</th><th data-i18n="view.crypto.th.from_ath">From ATH</th></tr></thead>
        <tbody>${rows.map(r => `
            <tr>
                <td>${r.market_cap_rank ?? '—'}</td>
                <td>${r.image ? `<img src="${esc(r.image)}" width="18" height="18" style="vertical-align:middle">` : ''}</td>
                <td><a href="#research/${encodeURIComponent(r.symbol.toUpperCase() + '-USD')}">
                    <strong>${esc(r.symbol.toUpperCase())}</strong></a>
                    <span class="muted small">${esc(r.name)}</span></td>
                <td>${r.current_price != null ? '$' + fmt(r.current_price, r.current_price < 1 ? 6 : 2) : '—'}</td>
                <td class="${(r.price_change_percentage_24h ?? 0) >= 0 ? 'pos' : 'neg'}">${pct(r.price_change_percentage_24h)}</td>
                <td class="${(r.price_change_percentage_7d_in_currency ?? 0) >= 0 ? 'pos' : 'neg'}">${pct(r.price_change_percentage_7d_in_currency)}</td>
                <td>${compact(r.market_cap)}</td>
                <td>${compact(r.total_volume)}</td>
                <td>${num(r.circulating_supply)} ${esc(r.symbol.toUpperCase())}</td>
                <td>${r.ath != null ? '$' + fmt(r.ath, r.ath < 1 ? 6 : 2) : '—'}</td>
                <td class="${(r.ath_change_percentage ?? 0) >= 0 ? 'pos' : 'neg'}">${pct(r.ath_change_percentage)}</td>
            </tr>`).join('')}</tbody></table>`;
}

function renderChain(c, mount) {
    const el = mount.querySelector('#c-onchain');
    if (!el) return;
    el.innerHTML = `
        <div class="card"><div class="label" data-i18n="view.crypto.card.hash_rate">Hash rate</div><div class="value">${c.hash_rate_thps != null ? (c.hash_rate_thps/1e6).toFixed(1)+' EH/s' : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.difficulty">Difficulty</div><div class="value">${c.difficulty != null ? (c.difficulty/1e12).toFixed(2)+'T' : '—'}</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.block_height">Block height</div><div class="value">${num(c.block_height)}</div></div>
        <div class="card"><div class="label" data-i18n="view.crypto.card.mempool_tx">Mempool tx</div><div class="value">${num(c.mempool_tx_count)}</div></div>
    `;
    try { applyUiI18n(el); } catch (_) {}
}
