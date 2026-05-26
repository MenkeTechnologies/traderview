import { api } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, md, pnlClass } from '../util.js';
import { ohlcChart } from '../charts.js';

const dtLocal = (iso) => {
    if (!iso) return '';
    const d = new Date(iso);
    const pad = (n) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
};

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
              <th>Time</th><th>Side</th><th>Qty</th><th>Price</th><th>Fee</th><th></th>
            </tr></thead><tbody>${executions.map(e => `
              <tr data-eid="${e.id}">
                <td><input class="ex-time" type="datetime-local"
                      value="${dtLocal(e.executed_at)}"></td>
                <td>
                  <select class="ex-side">
                    ${['buy','sell','short','cover'].map(s =>
                      `<option ${s === e.side ? 'selected' : ''}>${s}</option>`).join('')}
                  </select>
                </td>
                <td><input class="ex-qty" type="number" step="any" value="${e.qty}"></td>
                <td><input class="ex-price" type="number" step="any" value="${e.price}"></td>
                <td><input class="ex-fee" type="number" step="any" value="${e.fee}"></td>
                <td>
                  <button class="link" data-save-ex="${e.id}">save</button>
                  <button class="link" data-del-ex="${e.id}">trash</button>
                </td>
              </tr>
            `).join('')}</tbody></table>
            <details class="ex-add">
              <summary>+ Add execution</summary>
              <form id="ex-add-form" class="inline-form" style="margin-top:8px">
                <select name="side">
                  <option value="buy">buy</option><option value="sell">sell</option>
                  <option value="short">short</option><option value="cover">cover</option>
                </select>
                <input name="qty" type="number" step="any" placeholder="qty" required>
                <input name="price" type="number" step="any" placeholder="price" required>
                <input name="fee" type="number" step="any" placeholder="fee" value="0">
                <input name="executed_at" type="datetime-local" required>
                <button class="primary" type="submit">Add</button>
              </form>
            </details>
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
            <div class="inline-form">
              <button class="primary" id="journal-save">Save note</button>
              <button class="primary" id="journal-template" style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">Insert template</button>
            </div>
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
    document.getElementById('journal-template').addEventListener('click', async () => {
        const tpl = await api.defaultNoteTemplate('trade');
        const ta = document.getElementById('journal-body');
        if (tpl && tpl.body_md) {
            ta.value = (ta.value ? ta.value + '\n\n' : '') + tpl.body_md;
        } else {
            alert('No default trade template set. Configure one under Settings → Notes Templates.');
        }
    });
    document.querySelectorAll('[data-del-journal]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteJournal(b.dataset.delJournal);
            renderTradeDetail(mount, state, tradeId);
        }));

    // Execution editor — save / delete each row + add new
    document.querySelectorAll('[data-save-ex]').forEach(b =>
        b.addEventListener('click', async () => {
            const eid = b.dataset.saveEx;
            const row = b.closest('tr');
            const body = {
                side: row.querySelector('.ex-side').value,
                qty: Number(row.querySelector('.ex-qty').value),
                price: Number(row.querySelector('.ex-price').value),
                fee: Number(row.querySelector('.ex-fee').value),
                executed_at: new Date(row.querySelector('.ex-time').value).toISOString(),
            };
            try {
                await api.updateExecution(eid, body);
                renderTradeDetail(mount, state, tradeId);
            } catch (err) { alert('Save failed: ' + err.message); }
        }));
    document.querySelectorAll('[data-del-ex]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!confirm('Delete this execution? The trade will re-FIFO.')) return;
            await api.deleteExecution(b.dataset.delEx);
            renderTradeDetail(mount, state, tradeId);
        }));
    const addForm = document.getElementById('ex-add-form');
    if (addForm) {
        // pre-fill time with the trade's last close (or now)
        const dt = trade.closed_at || trade.opened_at;
        const d = new Date(dt);
        const pad = (n) => String(n).padStart(2, '0');
        addForm.querySelector('[name=executed_at]').value =
            `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
        addForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const fd = new FormData(e.target);
            const body = {
                side: fd.get('side'),
                qty: Number(fd.get('qty')),
                price: Number(fd.get('price')),
                fee: Number(fd.get('fee') || 0),
                executed_at: new Date(fd.get('executed_at')).toISOString(),
                asset_class: trade.asset_class,
                option_type: trade.option_type,
                strike: trade.strike,
                expiration: trade.expiration,
                multiplier: Number(trade.multiplier),
            };
            try {
                await api.addExecutionToTrade(tradeId, body);
                renderTradeDetail(mount, state, tradeId);
            } catch (err) { alert('Add failed: ' + err.message); }
        });
    }

    // Share
    document.getElementById('share-btn').addEventListener('click', async () => {
        const sh = await api.createShare({ trade_id: tradeId });
        document.getElementById('share-result').innerHTML =
            `Public link: <a href="#shared/${sh.slug}">/#shared/${sh.slug}</a> (slug: <code>${sh.slug}</code>)`;
    });
    void share;
}
