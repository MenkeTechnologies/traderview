import { api } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, md, pnlClass } from '../util.js';
import { ohlcChart } from '../charts.js';

export async function renderTradeDetail(mount, state, tradeId) {
    if (!tradeId) { mount.innerHTML = '<p class="boot">No trade id</p>'; return; }
    const [trade, executions, tags, journal, screenshots, share] = await Promise.all([
        api.trade(tradeId),
        api.executionsForTrade(tradeId),
        api.tagsForTrade(tradeId),
        api.journalForTrade(tradeId),
        api.screenshotsForTrade(tradeId),
        Promise.resolve(null),
    ]);

    mount.innerHTML = `
        <h1 class="view-title">// ${esc(trade.symbol)} · ${trade.side} · ${trade.status}</h1>
        <div class="cards">
            <div class="card"><div class="label">Net P&L</div>
                <div class="value ${pnlClass(trade.net_pnl)}">${fmtMoney(trade.net_pnl)}</div></div>
            <div class="card"><div class="label">Qty</div><div class="value">${fmt(trade.qty, 0)}</div></div>
            <div class="card"><div class="label">Entry / Exit</div>
                <div class="value">${fmt(trade.entry_avg)} → ${trade.exit_avg !== null ? fmt(trade.exit_avg) : '—'}</div></div>
            <div class="card"><div class="label">Fees</div><div class="value">${fmtMoney(trade.fees)}</div></div>
            <div class="card"><div class="label">MFE / MAE</div>
                <div class="value">${trade.mfe !== null ? fmtMoney(trade.mfe) : '—'} /
                ${trade.mae !== null ? fmtMoney(trade.mae) : '—'}</div></div>
            <div class="card"><div class="label">Best exit</div>
                <div class="value">${trade.best_exit_pnl !== null ? fmtMoney(trade.best_exit_pnl) : '—'}</div></div>
            <div class="card"><div class="label">Exit eff.</div>
                <div class="value">${trade.exit_efficiency !== null ? (Number(trade.exit_efficiency)*100).toFixed(1)+'%' : '—'}</div></div>
            <div class="card"><div class="label">Risk amount</div>
                <div class="value">${trade.risk_amount !== null ? fmtMoney(trade.risk_amount) : '—'}</div></div>
        </div>

        <div class="chart-panel">
            <h2>Chart</h2>
            <div id="chart-wrap"></div>
        </div>

        <div class="panel-grid">
          <div class="chart-panel">
            <h2>Executions</h2>
            <table class="trades"><thead><tr>
              <th>Time</th><th>Side</th><th>Qty</th><th>Price</th><th>Fee</th>
            </tr></thead><tbody>${executions.map(e => `
              <tr><td>${fmtDateTime(e.executed_at)}</td><td>${e.side}</td>
              <td>${fmt(e.qty, 0)}</td><td>${fmt(e.price)}</td><td>${fmtMoney(e.fee)}</td></tr>
            `).join('')}</tbody></table>
          </div>

          <div class="chart-panel">
            <h2>Tags</h2>
            <div class="tag-wrap" id="tags-wrap">
              ${tags.map(t => `<span class="tag-chip" style="border-color:${esc(t.color)}">${esc(t.name)}</span>`).join('')}
            </div>
            <div class="tag-add">
              <select id="tag-add-select"></select>
              <button class="primary" id="tag-add-btn">Add</button>
            </div>
          </div>

          <div class="chart-panel">
            <h2>Risk Plan</h2>
            <form id="risk-form" class="risk-form">
              <label>Stop loss <input name="stop_loss" type="number" step="any" value="${trade.stop_loss ?? ''}"></label>
              <label>Risk $ <input name="risk_amount" type="number" step="any" value="${trade.risk_amount ?? ''}"></label>
              <label>Target <input name="initial_target" type="number" step="any" value="${trade.initial_target ?? ''}"></label>
              <button class="primary" type="submit">Save</button>
            </form>
          </div>

          <div class="chart-panel">
            <h2>Screenshots</h2>
            <div class="screenshots" id="screenshots">
              ${screenshots.map(s => `
                <figure class="shot">
                  <img src="${api.screenshotUrl(s.id)}" alt="${esc(s.filename)}">
                  <figcaption>${esc(s.caption || s.filename)}
                    <button class="link" data-del="${s.id}">delete</button>
                  </figcaption>
                </figure>`).join('')}
            </div>
            <input type="file" id="shot-input" accept="image/*">
            <input type="text" id="shot-caption" placeholder="caption (optional)">
            <button class="primary" id="shot-upload">Upload</button>
          </div>

          <div class="chart-panel" style="grid-column: 1 / -1;">
            <h2>Journal — per-trade</h2>
            <div id="journal-list">${journal.map(j => `
              <div class="journal-entry">
                <div class="meta">${fmtDateTime(j.created_at)}</div>
                <div class="body">${md(j.body_md)}</div>
                <button class="link" data-del-journal="${j.id}">delete</button>
              </div>
            `).join('')}</div>
            <textarea id="journal-body" placeholder="What was the setup? What did you see? Mistakes? Lessons?"></textarea>
            <button class="primary" id="journal-save">Save note</button>
          </div>

          <div class="chart-panel">
            <h2>Share publicly</h2>
            <button class="primary" id="share-btn">Create share link</button>
            <div id="share-result"></div>
          </div>
        </div>
    `;

    // Chart — fetch ~ 5 trading days around the trade window.
    const opened = new Date(trade.opened_at).getTime() / 1000;
    const closed = trade.closed_at ? new Date(trade.closed_at).getTime() / 1000 : opened + 24*3600;
    const span = closed - opened;
    const interval = span < 3600 ? '5m' : span < 86400 ? '15m' : '1d';
    const padding = Math.max(span * 0.5, 3600);
    const bars = await api.bars(trade.symbol, interval,
        Math.floor(opened - padding), Math.floor(closed + padding))
        .catch(_ => ({ bars: [] }));
    const marks = executions.map(e => ({
        x: new Date(e.executed_at).getTime() / 1000,
        y: Number(e.price),
        side: e.side === 'buy' || e.side === 'cover' ? 'buy' : 'sell',
    }));
    ohlcChart(document.getElementById('chart-wrap'), bars.bars || [], marks, { height: 360 });

    // Tag add
    const allTags = await api.tags();
    const sel = document.getElementById('tag-add-select');
    const have = new Set(tags.map(t => t.id));
    sel.innerHTML = allTags.filter(t => !have.has(t.id))
        .map(t => `<option value="${t.id}">${esc(t.name)}</option>`).join('');
    document.getElementById('tag-add-btn').addEventListener('click', async () => {
        if (!sel.value) return;
        await api.attachTag(tradeId, sel.value);
        renderTradeDetail(mount, state, tradeId);
    });

    // Risk form
    document.getElementById('risk-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {};
        for (const k of ['stop_loss', 'risk_amount', 'initial_target']) {
            const v = fd.get(k);
            body[k] = v ? Number(v) : null;
        }
        await api.setRisk(tradeId, body);
        renderTradeDetail(mount, state, tradeId);
    });

    // Screenshot upload + delete
    document.getElementById('shot-upload').addEventListener('click', async () => {
        const file = document.getElementById('shot-input').files[0];
        if (!file) return;
        const cap = document.getElementById('shot-caption').value;
        await api.uploadScreenshot(tradeId, file, cap);
        renderTradeDetail(mount, state, tradeId);
    });
    document.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteScreenshot(b.dataset.del);
            renderTradeDetail(mount, state, tradeId);
        }));

    // Journal save / delete
    document.getElementById('journal-save').addEventListener('click', async () => {
        const body_md = document.getElementById('journal-body').value;
        if (!body_md.trim()) return;
        await api.createJournal({ trade_id: tradeId, body_md });
        renderTradeDetail(mount, state, tradeId);
    });
    document.querySelectorAll('[data-del-journal]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteJournal(b.dataset.delJournal);
            renderTradeDetail(mount, state, tradeId);
        }));

    // Share
    document.getElementById('share-btn').addEventListener('click', async () => {
        const sh = await api.createShare({ trade_id: tradeId });
        document.getElementById('share-result').innerHTML =
            `Public link: <a href="#shared/${sh.slug}">/#shared/${sh.slug}</a> (slug: <code>${sh.slug}</code>)`;
    });
    void share;
}
