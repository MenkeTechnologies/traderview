import { api } from '../api.js';
import { esc, fmt, fmtMoney, fmtDateTime, md, pnlClass } from '../util.js';
import { ohlcChart } from '../charts.js';
import { renderAiAnalyze } from './journal_ai.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const dtLocal = (iso) => {
    if (!iso) return '';
    const d = new Date(iso);
    const pad = (n) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
};

export async function renderTradeDetail(mount, state, tradeId) {
    const tok = currentViewToken();
    if (!tradeId) { mount.innerHTML = '<p data-i18n="view.trade_detail.hint.no_trade_id" class="boot">No trade id</p>'; return; }
    const [trade, executions, tags, journal, screenshots, share] = await Promise.all([
        api.trade(tradeId),
        api.executionsForTrade(tradeId),
        api.tagsForTrade(tradeId),
        api.journalForTrade(tradeId),
        api.screenshotsForTrade(tradeId),
        Promise.resolve(null),
    ]);
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <h1 class="view-title">// ${esc(trade.symbol)} · ${trade.side} · ${trade.status}</h1>
        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.net_pnl">Net P&L</div>
                <div class="value ${pnlClass(trade.net_pnl)}">${fmtMoney(trade.net_pnl)}</div></div>
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.qty">Qty</div><div class="value">${fmt(trade.qty, 0)}</div></div>
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.entry_exit">Entry / Exit</div>
                <div class="value">${fmt(trade.entry_avg)} → ${trade.exit_avg !== null ? fmt(trade.exit_avg) : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.fees">Fees</div><div class="value">${fmtMoney(trade.fees)}</div></div>
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.mfe_mae">MFE / MAE</div>
                <div class="value">${trade.mfe !== null ? fmtMoney(trade.mfe) : '—'} /
                ${trade.mae !== null ? fmtMoney(trade.mae) : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.best_exit">Best exit</div>
                <div class="value">${trade.best_exit_pnl !== null ? fmtMoney(trade.best_exit_pnl) : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.exit_eff">Exit eff.</div>
                <div class="value">${trade.exit_efficiency !== null ? (Number(trade.exit_efficiency)*100).toFixed(1)+'%' : '—'}</div></div>
            <div class="card"><div class="label" data-i18n="view.trade_detail.card.risk_amount">Risk amount</div>
                <div class="value">${trade.risk_amount !== null ? fmtMoney(trade.risk_amount) : '—'}</div></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.trade_detail.h2.chart">Chart</h2>
            <div id="chart-wrap"></div>
        </div>

        <div class="panel-grid">
          <div class="chart-panel">
            <h2 data-i18n="view.trade_detail.h2.executions">Executions</h2>
            <table class="trades"><thead><tr>
              <th data-i18n="view.trade_detail.th.time">Time</th><th data-i18n="view.trade_detail.th.side">Side</th><th data-i18n="view.trade_detail.th.qty">Qty</th><th data-i18n="view.trade_detail.th.price">Price</th><th data-i18n="view.trade_detail.th.fee">Fee</th><th></th>
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
                  <button data-i18n="view.trade_detail.btn.save" class="link" data-save-ex="${e.id}">save</button>
                  <button data-i18n="view.trade_detail.btn.trash" class="link" data-del-ex="${e.id}">trash</button>
                </td>
              </tr>
            `).join('')}</tbody></table>
            <details class="ex-add">
              <summary>+ Add execution</summary>
              <form id="ex-add-form" class="inline-form" style="margin-top:8px">
                <select name="side">
                  <option data-i18n="view.trade_detail.opt.buy" value="buy">buy</option><option data-i18n="view.trade_detail.opt.sell" value="sell">sell</option>
                  <option data-i18n="view.trade_detail.opt.short" value="short">short</option><option data-i18n="view.trade_detail.opt.cover" value="cover">cover</option>
                </select>
                <input name="qty" type="number" step="any" placeholder="qty" required>
                <input name="price" type="number" step="any" placeholder="price" required>
                <input name="fee" type="number" step="any" placeholder="fee" value="0">
                <input name="executed_at" type="datetime-local" required>
                <button data-i18n="view.trade_detail.btn.add" class="primary" type="submit">Add</button>
              </form>
            </details>
          </div>

          <div class="chart-panel">
            <h2 data-i18n="view.trade_detail.h2.tags">Tags</h2>
            <div class="tag-wrap" id="tags-wrap">
              ${tags.map(t => `<span class="tag-chip" style="border-color:${esc(t.color)}">${esc(t.name)}</span>`).join('')}
            </div>
            <div class="tag-add">
              <select id="tag-add-select"></select>
              <button data-i18n="view.trade_detail.btn.add_2" class="primary" id="tag-add-btn">Add</button>
            </div>
          </div>

          <div class="chart-panel">
            <h2 data-i18n="view.trade_detail.h2.risk_plan">Risk Plan</h2>
            <form id="risk-form" class="risk-form">
              <label><span data-i18n="view.trade_detail.label.stop_loss">Stop loss</span>
                  <input name="stop_loss" type="number" step="any" value="${trade.stop_loss ?? ''}"></label>
              <label><span data-i18n="view.trade_detail.label.risk_amount">Risk $</span>
                  <input name="risk_amount" type="number" step="any" value="${trade.risk_amount ?? ''}"></label>
              <label><span data-i18n="view.trade_detail.label.target">Target</span>
                  <input name="initial_target" type="number" step="any" value="${trade.initial_target ?? ''}"></label>
              <button data-i18n="view.trade_detail.btn.save_2" class="primary" type="submit">Save</button>
            </form>
          </div>

          <div class="chart-panel">
            <h2 data-i18n="view.trade_detail.h2.screenshots">Screenshots</h2>
            <div class="screenshots" id="screenshots">
              ${screenshots.map(s => `
                <figure class="shot">
                  <img src="${api.screenshotUrl(s.id)}" alt="${esc(s.filename)}">
                  <figcaption>${esc(s.caption || s.filename)}
                    <button data-i18n="view.trade_detail.btn.delete" class="link" data-del="${s.id}">delete</button>
                  </figcaption>
                </figure>`).join('')}
            </div>
            <input type="file" id="shot-input" accept="image/*">
            <input type="text" id="shot-caption" placeholder="caption (optional)">
            <button data-i18n="view.trade_detail.btn.upload" class="primary" id="shot-upload">Upload</button>
          </div>

          <div class="chart-panel" style="grid-column: 1 / -1;">
            <h2 data-i18n="view.trade_detail.h2.journal_per_trade">Journal — per-trade</h2>
            <div id="journal-list">${journal.map(j => `
              <div class="journal-entry">
                <div class="meta">${fmtDateTime(j.created_at)}</div>
                <div class="body">${md(j.body_md)}</div>
                <button data-i18n="view.trade_detail.btn.delete_2" class="link" data-del-journal="${j.id}">delete</button>
              </div>
            `).join('')}</div>
            <textarea id="journal-body" placeholder="What was the setup? What did you see? Mistakes? Lessons?"></textarea>
            <div class="inline-form">
              <button data-i18n="view.trade_detail.btn.save_note" class="primary" id="journal-save">Save note</button>
              <button data-i18n="view.trade_detail.btn.insert_template" class="primary" id="journal-template" style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">Insert template</button>
            </div>
          </div>

          <div class="chart-panel">
            <h2 data-i18n="view.trade_detail.h2.share_publicly">Share publicly</h2>
            <button data-i18n="view.trade_detail.btn.create_share_link" class="primary" id="share-btn">Create share link</button>
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
    if (!viewIsCurrent(tok)) return;
    const marks = executions.map(e => ({
        x: new Date(e.executed_at).getTime() / 1000,
        y: Number(e.price),
        side: e.side === 'buy' || e.side === 'cover' ? 'buy' : 'sell',
    }));
    const chartWrap = mount.querySelector('#chart-wrap');
    if (chartWrap) ohlcChart(chartWrap, bars.bars || [], marks, { height: 360 });

    // Tag add
    const allTags = await api.tags();
    if (!viewIsCurrent(tok)) return;
    const sel = mount.querySelector('#tag-add-select');
    const have = new Set(tags.map(t => t.id));
    if (sel) sel.innerHTML = allTags.filter(t => !have.has(t.id))
        .map(t => `<option value="${t.id}">${esc(t.name)}</option>`).join('');
    const tagAddBtn = mount.querySelector('#tag-add-btn');
    if (tagAddBtn) tagAddBtn.addEventListener('click', async () => {
        if (!sel || !sel.value) return;
        await api.attachTag(tradeId, sel.value);
        if (!viewIsCurrent(tok)) return;
        renderTradeDetail(mount, state, tradeId);
    });

    // Risk form
    mount.querySelector('#risk-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {};
        for (const k of ['stop_loss', 'risk_amount', 'initial_target']) {
            const v = fd.get(k);
            body[k] = v ? Number(v) : null;
        }
        await api.setRisk(tradeId, body);
        if (!viewIsCurrent(tok)) return;
        renderTradeDetail(mount, state, tradeId);
    });

    // Screenshot upload + delete
    mount.querySelector('#shot-upload').addEventListener('click', async () => {
        const inp = mount.querySelector('#shot-input');
        const file = inp && inp.files[0];
        if (!file) return;
        const capEl = mount.querySelector('#shot-caption');
        const cap = capEl ? capEl.value : '';
        await api.uploadScreenshot(tradeId, file, cap);
        if (!viewIsCurrent(tok)) return;
        renderTradeDetail(mount, state, tradeId);
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteScreenshot(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderTradeDetail(mount, state, tradeId);
        }));

    // Journal save / delete
    mount.querySelector('#journal-save').addEventListener('click', async () => {
        const ta = mount.querySelector('#journal-body');
        const body_md = ta ? ta.value : '';
        if (!body_md.trim()) return;
        await api.createJournal({ trade_id: tradeId, body_md });
        if (!viewIsCurrent(tok)) return;
        renderTradeDetail(mount, state, tradeId);
    });
    mount.querySelector('#journal-template').addEventListener('click', async () => {
        const tpl = await api.defaultNoteTemplate('trade');
        if (!viewIsCurrent(tok)) return;
        const ta = mount.querySelector('#journal-body');
        if (!ta) return;
        if (tpl && tpl.body_md) {
            ta.value = (ta.value ? ta.value + '\n\n' : '') + tpl.body_md;
        } else {
            alert('No default trade template set. Configure one under Settings → Notes Templates.');
        }
    });
    mount.querySelectorAll('[data-del-journal]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteJournal(b.dataset.delJournal);
            if (!viewIsCurrent(tok)) return;
            renderTradeDetail(mount, state, tradeId);
        }));

    // Execution editor — save / delete each row + add new
    mount.querySelectorAll('[data-save-ex]').forEach(b =>
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
                if (!viewIsCurrent(tok)) return;
                renderTradeDetail(mount, state, tradeId);
            } catch (err) { alert('Save failed: ' + err.message); }
        }));
    mount.querySelectorAll('[data-del-ex]').forEach(b =>
        b.addEventListener('click', async () => {
            if (!confirm(t('view.trade_detail.confirm.delete_execution'))) return;
            await api.deleteExecution(b.dataset.delEx);
            if (!viewIsCurrent(tok)) return;
            renderTradeDetail(mount, state, tradeId);
        }));
    const addForm = mount.querySelector('#ex-add-form');
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
                if (!viewIsCurrent(tok)) return;
                renderTradeDetail(mount, state, tradeId);
            } catch (err) { alert('Add failed: ' + err.message); }
        });
    }

    // Share
    mount.querySelector('#share-btn').addEventListener('click', async () => {
        const sh = await api.createShare({ trade_id: tradeId });
        if (!viewIsCurrent(tok)) return;
        const result = mount.querySelector('#share-result');
        if (result) result.innerHTML =
            `Public link: <a href="#shared/${sh.slug}">/#shared/${sh.slug}</a> (slug: <code>${sh.slug}</code>)`;
    });
    void share;

    // AI analysis panel — appended at the end. Self-contained: loads cache,
    // shows Run/Re-analyze button, fetches LLM on demand.
    const aiSlot = document.createElement('div');
    mount.appendChild(aiSlot);
    renderAiAnalyze(aiSlot, tradeId).catch((e) => {
        aiSlot.innerHTML = `<p class="muted small">${esc(t('view.trade_detail.ai_error', { msg: e.message }))}</p>`;
    });
}
