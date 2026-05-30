import { api } from '../api.js';
import { esc, fmtDate, fmtDateTime, fmtMoney, pnlClass } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

export async function renderSearch(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.search.h1.search" class="view-title">// SEARCH</h1>
        <form id="search-form" class="inline-form" style="margin-bottom:14px">
            <input name="q" data-shortcut="focus_search" data-tip="view.search.tip.q" placeholder="symbol, journal text, forum post…" data-i18n-placeholder="view.search.placeholder.q" autofocus required style="min-width:300px">
            <select name="scope" data-tip="view.search.tip.scope">
                <option data-i18n="view.search.opt.all" value="all">all</option>
                <option data-i18n="view.search.opt.trades" value="trades">trades</option>
                <option data-i18n="view.search.opt.journal" value="journal">journal</option>
                <option data-i18n="view.search.opt.forum" value="forum">forum</option>
            </select>
            <button data-i18n="view.search.btn.search" data-tip="view.search.tip.search" class="primary" type="submit">Search</button>
        </form>
        <div id="search-results"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.search.h2.hits_chart">Hits by category</h2>
            <div id="se-chart" style="width:100%;height:220px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.search.h2.rank_chart">FTS rank: journal + forum results</h2>
            <div id="se-rank-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    mount.querySelector('#search-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const q = fd.get('q');
        const scope = fd.get('scope');
        const el = mount.querySelector('#search-results');
        if (!el) return;
        el.innerHTML = '<div class="boot" data-i18n="common.status.searching">searching…</div>';
        try {
            const r = await api.search(q, scope);
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#search-results');
            if (elNow) elNow.innerHTML = renderHits(r);
            renderHitsChart(r);
            renderRankChart(r);
            const total = (r.trades || []).length + (r.journal || []).length + (r.forum || []).length;
            showToast(t('view.search.toast.done', { total, query: q }), { level: total > 0 ? 'success' : 'info' });
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#search-results');
            if (elNow) elNow.innerHTML = `<p class="boot">${err.message}</p>`;
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
}

function renderRankChart(r) {
    const el = document.getElementById('se-rank-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const journal = (r.journal || []).filter(j => Number.isFinite(Number(j.rank)));
    const forum   = (r.forum   || []).filter(f => Number.isFinite(Number(f.rank)));
    if (journal.length + forum.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.search.empty_rank_chart">${esc(t('view.search.empty_rank_chart'))}</div>`;
        return;
    }
    journal.sort((a, b) => Number(b.rank) - Number(a.rank));
    forum.sort((a, b) => Number(b.rank) - Number(a.rank));
    const rows = [
        ...journal.map(j => ({ rank: Number(j.rank), kind: 'journal' })),
        ...forum.map(f => ({ rank: Number(f.rank), kind: 'forum' })),
    ];
    const xs = rows.map((_, i) => i + 1);
    const jY = rows.map(r => r.kind === 'journal' ? r.rank : null);
    const fY = rows.map(r => r.kind === 'forum'   ? r.rank : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.search.chart.idx') },
            { label: t('view.search.chart.journal'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.search.chart.forum'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 10, fill: '#ffd84a', stroke: '#ffd84a' } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, jY, fY], el);
}

function renderHitsChart(r) {
    const el = document.getElementById('se-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const labels = [
        t('view.search.chart.trades'),
        t('view.search.chart.journal'),
        t('view.search.chart.forum'),
    ];
    const ys = [
        Number((r.trades || []).length),
        Number((r.journal || []).length),
        Number((r.forum || []).length),
    ];
    if (ys.reduce((a, b) => a + b, 0) < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.search.empty_chart">${esc(t('view.search.empty_chart'))}</div>`;
        return;
    }
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.search.chart.scope') },
            { label: t('view.search.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderHits(r) {
    const blocks = [];
    if (r.trades.length) {
        blocks.push(`
            <div class="chart-panel">
              <h2>${esc(t('view.search.h2.trades', { count: r.trades.length }))}</h2>
              <table class="trades">
                <thead><tr><th data-i18n="view.search.th.symbol">Symbol</th><th data-i18n="view.search.th.side">Side</th><th data-i18n="view.search.th.status">Status</th><th data-i18n="view.search.th.opened">Opened</th><th data-i18n="view.search.th.net_p_l">Net P&L</th></tr></thead>
                <tbody>${r.trades.map(t => `
                    <tr data-context-scope="trade-row" data-id="${esc(t.id)}">
                      <td><a href="#trade/${t.id}">${esc(t.symbol)}</a></td>
                      <td>${t.side}</td><td>${t.status}</td>
                      <td>${fmtDateTime(t.opened_at)}</td>
                      <td class="${pnlClass(t.net_pnl)}">${t.net_pnl !== null ? fmtMoney(t.net_pnl) : '—'}</td>
                    </tr>`).join('')}</tbody>
              </table>
            </div>
        `);
    }
    if (r.journal.length) {
        blocks.push(`
            <div class="chart-panel">
              <h2>${esc(t('view.search.h2.journal', { count: r.journal.length }))}</h2>
              ${r.journal.map(j => `
                <div class="journal-entry"
                     data-context-scope="journal-entry"
                     data-id="${esc(j.id)}"
                     data-trade-id="${esc(j.trade_id || '')}">
                  <div class="meta">
                    ${j.day ? fmtDate(j.day) : fmtDateTime(j.created_at)}
                    ${j.trade_id ? `· <a href="#trade/${j.trade_id}">${esc(t('common.link.trade'))}</a>` : ''}
                    · rank ${j.rank.toFixed(3)}
                  </div>
                  <div class="body">${j.snippet}</div>
                </div>`).join('')}
            </div>
        `);
    }
    if (r.forum.length) {
        blocks.push(`
            <div class="chart-panel">
              <h2>${esc(t('view.search.h2.forum', { count: r.forum.length }))}</h2>
              ${r.forum.map(f => `
                <div class="forum-post">
                  <div class="meta">
                    <a href="#community/${f.category_slug}/${f.thread_slug}">${esc(f.thread_title)}</a>
                    · ${fmtDateTime(f.created_at)} · rank ${f.rank.toFixed(3)}
                  </div>
                  <div class="body">${f.snippet}</div>
                </div>`).join('')}
            </div>
        `);
    }
    return blocks.length
        ? blocks.join('')
        : `<p class="boot">${esc(t('view.search.no_matches', { query: r.query }))}</p>`;
}
