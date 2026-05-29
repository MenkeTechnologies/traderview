// Expenses view — business expense tracking + Schedule C mapping.
//
// Layout:
//   ┌─ toolbar: account picker, source picker, csv upload, rules button ─┐
//   ├─ filter row: date range, category, business toggle ────────────────┤
//   ├─ transaction table (inline category dropdown per row) ─────────────┤
//   └─ rules modal (lazy) ─────────────────────────────────────────────────┘
//
// CSV parsers are stubs until real samples arrive — upload returns 400 with
// the parser-stub message, which we surface verbatim.

import { api, ApiError } from '../api.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const state = {
    accounts: [],
    categories: [],
    currentAccountId: '',     // '' = ALL
    filters: { from: '', to: '', category: '', is_business: '', search: '' },
    transactions: [],
    mount: null,
    tok: 0,
};

export async function renderExpensesView(mount) {
    const tok = currentViewToken();
    state.mount = mount;
    state.tok = tok;
    mount.innerHTML = '<div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div>';
    try {
        const [accts, cats] = await Promise.all([
            api.expenseAccounts(),
            api.expenseCategories(),
        ]);
        if (!viewIsCurrent(tok)) return;
        state.accounts = accts;
        state.categories = cats;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        mount.innerHTML = `<p class="boot">expense load failed: ${e.message}</p>`;
        return;
    }
    drawShell(mount);
    await refresh();
}

function drawShell(mount) {
    const accountOpts = ['<option data-i18n="view.expenses.opt.all_accounts" value="">all accounts</option>']
        .concat(state.accounts.map(a => `<option value="${a.id}">${esc(a.name)} (${a.kind})</option>`))
        .join('');

    const sourceOpts = [
        ['amazon', 'Amazon (CSV / XLSX)'],
        ['bofa', 'Bank of America (CSV / XLSX)'],
        ['chase', 'Chase (CSV)'],
        ['apple_card', 'Apple Card (PDF)'],
    ].map(([v, l]) => `<option value="${v}">${l}</option>`).join('');

    const catOpts = state.categories
        .map(c => `<option value="${c.code}">${c.schedule_c_line}. ${esc(c.label)}</option>`)
        .join('');

    mount.innerHTML = `
    <div class="expense-toolbar">
        <select id="exp-account">${accountOpts}</select>
        <button data-i18n="view.expenses.btn.account" class="primary" id="exp-new-account">+ Account</button>
        <span class="sep"></span>
        <select id="exp-source">${sourceOpts}</select>
        <input type="file" id="exp-file" class="hidden"
               accept=".csv,.xlsx,.xls,.ods,.pdf,text/csv,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet,application/vnd.oasis.opendocument.spreadsheet,application/pdf">
        <button data-i18n="view.expenses.btn.upload_statement" class="primary" id="exp-upload">Upload statement</button>
        <span class="sep"></span>
        <button data-i18n="view.expenses.btn.seed_default_rules" id="exp-seed-rules">Seed default rules</button>
        <button data-i18n="view.expenses.btn.rules" id="exp-rules-btn">Rules</button>
        <button data-i18n="view.expenses.btn.receipts" id="exp-receipts-btn">Receipts</button>
        <button data-i18n="view.expenses.btn.schedule_c" id="exp-report-btn">Schedule C</button>
    </div>

    <div class="receipt-dropzone" id="receipt-dz">
        <strong>Drop receipt</strong> — JPG, PNG, WebP, or PDF.
        OCR runs in the background; results auto-match candidate transactions.
        <input type="file" id="receipt-file" class="hidden" accept="image/jpeg,image/png,image/webp,image/bmp,application/pdf">
    </div>

    <div class="expense-filters">
        <label><span data-i18n="view.expenses.label.from">From</span>
            <input type="date" id="exp-from"></label>
        <label><span data-i18n="view.expenses.label.to">To</span>
            <input type="date" id="exp-to"></label>
        <label><span data-i18n="view.expenses.label.category">Category</span>
            <select id="exp-category">
                <option data-i18n="view.expenses.opt.all" value="">all</option>
                <option data-i18n="view.expenses.opt.uncategorized" value="__none__">(uncategorized)</option>
                ${catOpts}
            </select>
        </label>
        <label><span data-i18n="view.expenses.label.business">Business</span>
            <select id="exp-business">
                <option data-i18n="view.expenses.opt.all_2" value="">all</option>
                <option data-i18n="view.expenses.opt.business_only" value="true">business only</option>
                <option data-i18n="view.expenses.opt.personal_only" value="false">personal only</option>
            </select>
        </label>
        <label><span data-i18n="view.expenses.label.search">Search</span>
            <input type="text" id="exp-search" placeholder="merchant / description"
                   data-i18n-placeholder="view.expenses.placeholder.search"></label>
        <button data-i18n="view.expenses.btn.apply" id="exp-apply">Apply</button>
    </div>

    <div id="exp-status" class="expense-status"></div>
    <div id="exp-table"></div>
    <div id="exp-rules-modal" class="modal hidden"></div>
    `;

    mount.querySelector('#exp-account').addEventListener('change', e => {
        state.currentAccountId = e.target.value;
        refresh();
    });
    mount.querySelector('#exp-new-account').addEventListener('click', createAccountFlow);
    mount.querySelector('#exp-upload').addEventListener('click', () => {
        mount.querySelector('#exp-file').click();
    });
    mount.querySelector('#exp-file').addEventListener('change', handleUpload);
    mount.querySelector('#exp-seed-rules').addEventListener('click', seedRulesFlow);
    mount.querySelector('#exp-rules-btn').addEventListener('click', openRulesModal);
    mount.querySelector('#exp-receipts-btn').addEventListener('click', openReceiptsModal);
    mount.querySelector('#exp-report-btn').addEventListener('click', () => openScheduleCModal());

    bindReceiptDropzone();
    mount.querySelector('#exp-apply').addEventListener('click', () => {
        state.filters.from = mount.querySelector('#exp-from').value;
        state.filters.to = mount.querySelector('#exp-to').value;
        const catVal = mount.querySelector('#exp-category').value;
        state.filters.category = catVal === '__none__' ? '__none__' : catVal;
        state.filters.is_business = mount.querySelector('#exp-business').value;
        state.filters.search = mount.querySelector('#exp-search').value;
        refresh();
    });
}

async function refresh() {
    const params = {
        limit: 500,
        ...(state.currentAccountId ? { account_id: state.currentAccountId } : {}),
        ...(state.filters.from ? { from: state.filters.from } : {}),
        ...(state.filters.to ? { to: state.filters.to } : {}),
        ...(state.filters.category && state.filters.category !== '__none__'
            ? { category: state.filters.category } : {}),
        ...(state.filters.is_business !== '' ? { is_business: state.filters.is_business } : {}),
        ...(state.filters.search ? { search: state.filters.search } : {}),
    };
    try {
        let rows = await api.expenseTransactions(params);
        if (!viewIsCurrent(state.tok)) return;
        if (state.filters.category === '__none__') {
            rows = rows.filter(r => !r.category_code);
        }
        state.transactions = rows;
        drawTable();
    } catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        const tbl = state.mount.querySelector('#exp-table');
        if (tbl) tbl.innerHTML = `<p class="boot">transactions load failed: ${esc(e.message)}</p>`;
    }
}

function drawTable() {
    const host = state.mount.querySelector('#exp-table');
    if (!host) return;
    if (!state.transactions.length) {
        host.innerHTML = `<p data-i18n="view.expenses.hint.no_transactions_upload_a_csv" class="boot">no transactions. upload a CSV.</p>`;
        return;
    }
    const acctNames = Object.fromEntries(state.accounts.map(a => [a.id, a.name]));

    const catOptsBase = '<option data-i18n="view.expenses.opt.uncategorized_2" value="">(uncategorized)</option>' +
        state.categories
            .map(c => `<option value="${c.code}">${c.schedule_c_line}. ${esc(c.label)}</option>`)
            .join('');

    const rows = state.transactions.map(t => {
        const amt = Number(t.amount);
        const cls = amt < 0 ? 'pnl-neg' : 'pnl-pos';
        const xferCls = t.is_transfer ? 'tx-transfer' : '';
        const bizClass = t.is_business ? 'biz-on' : 'biz-off';
        const catSel = state.categories
            .map(c =>
                `<option value="${c.code}"${c.code === t.category_code ? ' selected' : ''}>` +
                `${c.schedule_c_line}. ${esc(c.label)}</option>`)
            .join('');
        return `
        <tr class="${xferCls}" data-tx="${t.id}">
            <td>${t.posted_at.slice(0, 10)}</td>
            <td>${esc(acctNames[t.account_id] || t.account_id.slice(0, 8))}</td>
            <td title="${esc(t.merchant_raw)}">${esc(t.merchant_raw)}</td>
            <td class="${cls}">${amt.toFixed(2)}</td>
            <td>
                <select class="exp-cat" data-tx="${t.id}">
                    <option data-i18n="view.expenses.opt.uncategorized_3" value=""${t.category_code ? '' : ' selected'}>(uncategorized)</option>
                    ${catSel}
                </select>
            </td>
            <td>
                <button class="tx-biz ${bizClass}" data-tx="${t.id}" data-biz="${t.is_business}">
                    ${t.is_business ? 'BIZ' : 'pers'}
                </button>
            </td>
            <td>
                <button class="tx-xfer ${t.is_transfer ? 'biz-on' : ''}"
                        data-tx="${t.id}" data-xfer="${t.is_transfer}">
                    ${t.is_transfer ? 'XFER' : '—'}
                </button>
            </td>
        </tr>`;
    }).join('');

    host.innerHTML = `
        <table class="trades expense-table">
            <thead><tr>
                <th data-i18n="view.expenses.th.date">Date</th><th data-i18n="view.expenses.th.account">Account</th><th data-i18n="view.expenses.th.merchant">Merchant</th>
                <th data-i18n="view.expenses.th.amount">Amount</th><th data-i18n="view.expenses.th.category_schedule_c">Category (Schedule C)</th><th data-i18n="view.expenses.th.biz">Biz?</th><th data-i18n="view.expenses.th.transfer">Transfer?</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>`;

    host.querySelectorAll('select.exp-cat').forEach(sel => {
        sel.addEventListener('change', async ev => {
            const tx = ev.target.dataset.tx;
            const code = ev.target.value || null;
            try {
                await api.updateExpenseTransaction(tx, { category_code: code });
            } catch (e) {
                alert(`update failed: ${e.message}`);
            }
        });
    });
    host.querySelectorAll('button.tx-biz').forEach(btn => {
        btn.addEventListener('click', async () => {
            const tx = btn.dataset.tx;
            const next = btn.dataset.biz !== 'true';
            try {
                await api.updateExpenseTransaction(tx, { is_business: next });
                btn.dataset.biz = String(next);
                btn.textContent = next ? 'BIZ' : 'pers';
                btn.classList.toggle('biz-on', next);
                btn.classList.toggle('biz-off', !next);
            } catch (e) { alert(`update failed: ${e.message}`); }
        });
    });
    host.querySelectorAll('button.tx-xfer').forEach(btn => {
        btn.addEventListener('click', async () => {
            const tx = btn.dataset.tx;
            const next = btn.dataset.xfer !== 'true';
            try {
                await api.updateExpenseTransaction(tx, { is_transfer: next });
                btn.dataset.xfer = String(next);
                btn.textContent = next ? 'XFER' : '—';
                btn.classList.toggle('biz-on', next);
            } catch (e) { alert(`update failed: ${e.message}`); }
        });
    });
}

async function createAccountFlow() {
    const name = prompt('Account name (e.g. "BofA Business Checking"):');
    if (!name) return;
    const kind = prompt('Kind: bank | credit_card | marketplace', 'credit_card');
    if (!['bank', 'credit_card', 'marketplace'].includes(kind)) {
        alert('invalid kind');
        return;
    }
    const source = prompt('Source id (bofa | chase | apple_card | amazon | manual):', 'chase');
    if (!source) return;
    try {
        const acct = await api.createExpenseAccount({ kind, source, name });
        if (!viewIsCurrent(state.tok)) return;
        state.accounts.push(acct);
        state.currentAccountId = acct.id;
        // Redraw shell so the account picker re-renders with the new option.
        drawShell(state.mount);
        const sel = state.mount.querySelector('#exp-account');
        if (sel) sel.value = acct.id;
        await refresh();
    } catch (e) {
        alert(`create failed: ${e.message}`);
    }
}

async function handleUpload(ev) {
    const file = ev.target.files && ev.target.files[0];
    ev.target.value = '';
    if (!file) return;
    if (!state.currentAccountId) {
        alert('pick an account first (or create one)');
        return;
    }
    const source = state.mount.querySelector('#exp-source').value;
    const status = state.mount.querySelector('#exp-status');
    if (status) status.textContent = `uploading ${file.name}…`;
    try {
        const res = await api.importExpense(state.currentAccountId, source, file);
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (res.duplicate) {
            if (status2) status2.textContent = `already imported (sha matches existing import ${res.import_id.slice(0, 8)})`;
        } else {
            if (status2) status2.textContent =
                `imported ${res.inserted_count}/${res.row_count} rows · ` +
                `auto-categorized ${res.categorized_count} · ` +
                `transfer pairs ${res.transfer_pairs}`;
        }
        await refresh();
    } catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (status2) {
            if (e instanceof ApiError && e.status === 400) {
                status2.textContent = `parser: ${e.message}`;
            } else {
                status2.textContent = `upload failed: ${e.message}`;
            }
        }
    }
}

async function seedRulesFlow() {
    const status = state.mount.querySelector('#exp-status');
    try {
        const res = await api.seedExpenseRules();
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (status2) status2.textContent = res.skipped_existing
            ? `you already have ${res.skipped_existing} rules — seed skipped`
            : `seeded ${res.inserted} default rules`;
    } catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        const status2 = state.mount.querySelector('#exp-status');
        if (status2) status2.textContent = `seed failed: ${e.message}`;
    }
}

async function openRulesModal() {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    modal.innerHTML = '<div class="modal-inner"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>';
    let rules = [];
    try { rules = await api.expenseRules(); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">load failed: ${esc(e.message)}</p>
            <button data-i18n="view.expenses.btn.close" id="rules-close">Close</button></div>`;
        modal.querySelector('#rules-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;
    const catOpts = state.categories
        .map(c => `<option value="${c.code}">${c.schedule_c_line}. ${esc(c.label)}</option>`)
        .join('');

    const rows = rules.map(r => `
        <tr>
            <td>${r.priority}</td>
            <td>${esc(r.pattern)}</td>
            <td>${r.pattern_kind}</td>
            <td>${r.category_code}</td>
            <td>${r.is_business ? 'biz' : 'pers'}</td>
            <td>${r.hit_count}</td>
            <td><button data-i18n="view.expenses.btn.delete" class="rule-del" data-id="${r.id}">delete</button></td>
        </tr>`).join('');

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2 data-i18n="view.expenses.h2.merchant_rules">Merchant rules</h2>
        <form id="rule-form" class="rule-form">
            <input name="pattern" placeholder="pattern (e.g. uber)" required>
            <select name="pattern_kind">
                <option data-i18n="view.expenses.opt.substring" value="substring">substring</option>
                <option data-i18n="view.expenses.opt.regex" value="regex">regex</option>
            </select>
            <select name="category_code">${catOpts}</select>
            <label><input type="checkbox" name="is_business" checked> biz</label>
            <input name="priority" type="number" value="100" style="width:60px">
            <label><input type="checkbox" name="apply_retroactively" checked> apply now</label>
            <button data-i18n="view.expenses.btn.add" class="primary" type="submit">add</button>
        </form>
        <table class="trades">
            <thead><tr><th data-i18n="view.expenses.th.pri">Pri</th><th data-i18n="view.expenses.th.pattern">Pattern</th><th data-i18n="view.expenses.th.kind">Kind</th><th data-i18n="view.expenses.th.cat">Cat</th><th data-i18n="view.expenses.th.biz_2">Biz?</th><th data-i18n="view.expenses.th.hits">Hits</th><th></th></tr></thead>
            <tbody>${rows || '<tr><td colspan="7" class="boot">no rules yet</td></tr>'}</tbody>
        </table>
        <button data-i18n="view.expenses.btn.close_2" id="rules-close" style="margin-top:12px">Close</button>
    </div>`;
    modal.querySelector('#rules-close').onclick = () => modal.classList.add('hidden');
    modal.querySelectorAll('.rule-del').forEach(btn => {
        btn.onclick = async () => {
            if (!confirm('delete this rule?')) return;
            try { await api.deleteExpenseRule(btn.dataset.id); }
            catch (e) { alert(`delete failed: ${e.message}`); return; }
            if (!viewIsCurrent(state.tok)) return;
            openRulesModal();
        };
    });
    modal.querySelector('#rule-form').addEventListener('submit', async ev => {
        ev.preventDefault();
        const fd = new FormData(ev.target);
        try {
            await api.createExpenseRule({
                pattern: fd.get('pattern'),
                pattern_kind: fd.get('pattern_kind'),
                category_code: fd.get('category_code'),
                is_business: fd.get('is_business') === 'on',
                priority: Number(fd.get('priority')) || 100,
                apply_retroactively: fd.get('apply_retroactively') === 'on',
            });
            if (!viewIsCurrent(state.tok)) return;
            await refresh();
            openRulesModal();
        } catch (e) { alert(`create failed: ${e.message}`); }
    });
}

function esc(s) {
    return String(s == null ? '' : s)
        .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

// --- receipt drag-drop + OCR poll + match modal --------------------------

function bindReceiptDropzone() {
    const dz = state.mount.querySelector('#receipt-dz');
    const picker = state.mount.querySelector('#receipt-file');
    if (!dz || !picker) return;
    dz.addEventListener('click', () => picker.click());
    ['dragenter', 'dragover'].forEach(ev =>
        dz.addEventListener(ev, e => { e.preventDefault(); dz.classList.add('dragover'); }));
    ['dragleave', 'drop'].forEach(ev =>
        dz.addEventListener(ev, e => { e.preventDefault(); dz.classList.remove('dragover'); }));
    dz.addEventListener('drop', e => receiptUploadAll(e.dataTransfer.files));
    picker.addEventListener('change', () => {
        receiptUploadAll(picker.files);
        picker.value = '';
    });
}

async function receiptUploadAll(fileList) {
    if (!fileList || !fileList.length) return;
    const setStatus = (txt) => {
        const s = state.mount.querySelector('#exp-status');
        if (s) s.textContent = txt;
    };
    for (const file of fileList) {
        setStatus(`uploading ${file.name}…`);
        try {
            const r = await api.uploadReceipt(file);
            if (!viewIsCurrent(state.tok)) return;
            setStatus(`uploaded ${file.name} — OCR running…`);
            pollReceiptUntilReady(r.id);
        } catch (e) {
            if (!viewIsCurrent(state.tok)) return;
            setStatus(`receipt upload failed: ${e.message}`);
        }
    }
}

async function pollReceiptUntilReady(receiptId) {
    const setStatus = (txt) => {
        const s = state.mount.querySelector('#exp-status');
        if (s) s.textContent = txt;
    };
    const maxAttempts = 60;        // 60 * 2s = 2 min ceiling
    for (let i = 0; i < maxAttempts; i++) {
        await new Promise(r => setTimeout(r, 2000));
        if (!viewIsCurrent(state.tok)) return;
        let meta;
        try { meta = await api.receiptMeta(receiptId); }
        catch (e) {
            if (!viewIsCurrent(state.tok)) return;
            setStatus(`receipt poll failed: ${e.message}`);
            return;
        }
        if (!viewIsCurrent(state.tok)) return;
        if (meta.ocr_status === 'pending') continue;
        if (meta.ocr_status === 'failed') {
            setStatus(`OCR failed: ${meta.error_message || 'unknown'}`);
            return;
        }
        if (meta.ocr_status === 'needs_image') {
            setStatus(`OCR needs image: ${meta.error_message}`);
            return;
        }
        // done — open match suggestion modal
        setStatus(`OCR done: ${meta.ocr_merchant || '?'} · ${meta.ocr_total ?? '?'} · ${meta.ocr_date ?? '?'}`);
        openReceiptMatchModal(meta);
        return;
    }
    setStatus(`OCR timed out for receipt ${receiptId.slice(0, 8)}`);
}

async function openReceiptMatchModal(meta) {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    modal.innerHTML = '<div class="modal-inner"><p data-i18n="view.expenses.hint.scoring_candidates" class="boot">scoring candidates…</p></div>';
    let matches = [];
    try { matches = await api.receiptMatches(meta.id); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">match load failed: ${esc(e.message)}</p>
            <button data-i18n="view.expenses.btn.close_3" id="m-close">Close</button></div>`;
        modal.querySelector('#m-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;

    const acctNames = Object.fromEntries(state.accounts.map(a => [a.id, a.name]));

    const rows = matches.map(m => `
        <tr>
            <td>${(m.score * 100).toFixed(0)}%</td>
            <td>${m.transaction.posted_at.slice(0, 10)}</td>
            <td>${esc(acctNames[m.transaction.account_id] || '?')}</td>
            <td>${esc(m.transaction.merchant_raw)}</td>
            <td>${Number(m.transaction.amount).toFixed(2)}</td>
            <td><button data-i18n="view.expenses.btn.attach" class="m-pick primary" data-tx="${m.transaction.id}">attach</button></td>
        </tr>`).join('');

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2 data-i18n="view.expenses.h2.receipt_transaction">Receipt → transaction</h2>
        <div class="receipt-summary">
            <strong>Merchant:</strong> ${esc(meta.ocr_merchant || '?')} ·
            <strong>Total:</strong> ${meta.ocr_total ?? '?'} ·
            <strong>Date:</strong> ${meta.ocr_date ?? '?'} ·
            <strong>Conf:</strong> ${meta.ocr_confidence != null ? (meta.ocr_confidence * 100).toFixed(0) + '%' : '?'}
        </div>
        <table class="trades">
            <thead><tr><th data-i18n="view.expenses.th.score">Score</th><th data-i18n="view.expenses.th.date_2">Date</th><th data-i18n="view.expenses.th.account_2">Account</th><th data-i18n="view.expenses.th.merchant_2">Merchant</th><th data-i18n="view.expenses.th.amount_2">Amount</th><th></th></tr></thead>
            <tbody>${rows || '<tr><td colspan="6" class="boot">no candidates above threshold — attach manually from the receipts list</td></tr>'}</tbody>
        </table>
        <div style="margin-top:12px;display:flex;gap:8px">
            <a href="${api.receiptBlobUrl(meta.id)}" target="_blank">View receipt</a>
            <button data-i18n="view.expenses.btn.close_4" id="m-close" style="margin-left:auto">Close</button>
        </div>
    </div>`;
    modal.querySelector('#m-close').onclick = () => modal.classList.add('hidden');
    modal.querySelectorAll('.m-pick').forEach(btn => {
        btn.onclick = async () => {
            try {
                await api.attachReceipt(meta.id, btn.dataset.tx);
                if (!viewIsCurrent(state.tok)) return;
                modal.classList.add('hidden');
                const s = state.mount.querySelector('#exp-status');
                if (s) s.textContent = 'receipt attached';
                await refresh();
            } catch (e) { alert(`attach failed: ${e.message}`); }
        };
    });
}

async function openScheduleCModal(year) {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    const initialYear = year || new Date().getFullYear();
    modal.innerHTML = '<div class="modal-inner"><p data-i18n="view.expenses.hint.building_report" class="boot">building report…</p></div>';
    let report;
    try { report = await api.scheduleC(initialYear); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">report failed: ${esc(e.message)}</p>
            <button data-i18n="view.expenses.btn.close_5" id="sc-close">Close</button></div>`;
        modal.querySelector('#sc-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;

    const fmt = n => Number(n).toLocaleString(undefined, {
        minimumFractionDigits: 2, maximumFractionDigits: 2,
    });

    const lines = report.lines.map(l => {
        const ded = Number(l.deduction_pct);
        const pctLabel = ded === 1 ? '100%' : `${(ded * 100).toFixed(0)}%`;
        return `<tr>
            <td>${l.schedule_c_line}</td>
            <td>${esc(l.label)}</td>
            <td class="num">${fmt(l.raw_total)}</td>
            <td>${pctLabel}</td>
            <td class="num"><strong>${fmt(l.deductible_total)}</strong></td>
            <td class="num">${l.txn_count}</td>
        </tr>`;
    }).join('');

    // Year picker: current year ± 4.
    const now = new Date().getFullYear();
    const years = [];
    for (let y = now - 4; y <= now + 1; y++) years.push(y);
    const yearOpts = years.map(y =>
        `<option value="${y}"${y === report.year ? ' selected' : ''}>${y}</option>`).join('');

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2>${esc(t('view.expenses.h2.schedule_c', { year: report.year }))}</h2>
        <div style="display:flex;gap:12px;margin-bottom:12px;align-items:center">
            <label><span data-i18n="view.expenses.label.year">Year</span>
                <select id="sc-year">${yearOpts}</select></label>
            <span style="color:var(--fg-2);font-size:11px">
                window: ${report.from_date} → ${report.to_date} ·
                excluded: ${report.excluded_transfers} transfers, ${report.excluded_personal} personal
            </span>
        </div>
        <table class="trades sc-table">
            <thead><tr>
                <th data-i18n="view.expenses.th.line">Line</th><th data-i18n="view.expenses.th.category">Category</th>
                <th data-i18n="view.expenses.th.raw" class="num">Raw $</th><th data-i18n="view.expenses.th.ded">Ded %</th>
                <th data-i18n="view.expenses.th.deductible" class="num">Deductible $</th><th class="num">#</th>
            </tr></thead>
            <tbody>${lines}</tbody>
            <tfoot>
                <tr>
                    <td colspan="2"><strong>Grand total (categorized business)</strong></td>
                    <td class="num">${fmt(report.grand_total_raw)}</td>
                    <td></td>
                    <td class="num"><strong>${fmt(report.grand_total_deductible)}</strong></td>
                    <td></td>
                </tr>
                <tr>
                    <td colspan="2" style="color:var(--yellow)">⚠ Uncategorized business expenses</td>
                    <td class="num" style="color:var(--yellow)">${fmt(report.uncategorized_total)}</td>
                    <td></td>
                    <td></td>
                    <td class="num">${report.uncategorized_count}</td>
                </tr>
            </tfoot>
        </table>
        <p style="color:var(--fg-2);font-size:11px;margin-top:12px">
            Deductible column applies each category's IRS rate (meals 50%; everything else 100%).
            Uncategorized business expenses do <strong>not</strong> roll into the grand total —
            tag them in the transaction list first.
        </p>
        <button data-i18n="view.expenses.btn.close_6" id="sc-close" style="margin-top:8px">Close</button>
    </div>`;
    modal.querySelector('#sc-close').onclick = () => modal.classList.add('hidden');
    modal.querySelector('#sc-year').addEventListener('change', e => {
        openScheduleCModal(Number(e.target.value));
    });
}

async function openReceiptsModal() {
    const modal = state.mount.querySelector('#exp-rules-modal');
    if (!modal) return;
    modal.classList.remove('hidden');
    modal.innerHTML = '<div class="modal-inner"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text">loading…</div></div></div>';
    let rs = [];
    try { rs = await api.receipts(); }
    catch (e) {
        if (!viewIsCurrent(state.tok)) return;
        modal.innerHTML = `<div class="modal-inner"><p class="boot">load failed: ${esc(e.message)}</p>
            <button data-i18n="view.expenses.btn.close_7" id="r-close">Close</button></div>`;
        modal.querySelector('#r-close').onclick = () => modal.classList.add('hidden');
        return;
    }
    if (!viewIsCurrent(state.tok)) return;

    const rows = rs.map(r => `
        <tr>
            <td>${r.created_at.slice(0, 10)}</td>
            <td>${esc(r.filename)}</td>
            <td>${r.ocr_status}</td>
            <td>${esc(r.ocr_merchant || '')}</td>
            <td>${r.ocr_total ?? ''}</td>
            <td>${r.ocr_date ?? ''}</td>
            <td>${r.transaction_id ? r.transaction_id.slice(0, 8) : ''}</td>
            <td>
                <a href="${api.receiptBlobUrl(r.id)}" target="_blank">view</a>
                ${!r.transaction_id && r.ocr_status === 'done' ? ` · <button data-i18n="view.expenses.btn.match" class="r-match" data-id="${r.id}">match</button>` : ''}
            </td>
        </tr>`).join('');

    modal.innerHTML = `
    <div class="modal-inner wide">
        <h2 data-i18n="view.expenses.h2.receipts">Receipts</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.expenses.th.uploaded">Uploaded</th><th data-i18n="view.expenses.th.filename">Filename</th><th data-i18n="view.expenses.th.ocr">OCR</th>
                <th data-i18n="view.expenses.th.merchant_3">Merchant</th><th data-i18n="view.expenses.th.total">Total</th><th data-i18n="view.expenses.th.date_3">Date</th><th data-i18n="view.expenses.th.tx">Tx</th><th></th>
            </tr></thead>
            <tbody>${rows || '<tr><td colspan="8" class="boot">no receipts uploaded</td></tr>'}</tbody>
        </table>
        <button data-i18n="view.expenses.btn.close_8" id="r-close" style="margin-top:12px">Close</button>
    </div>`;
    modal.querySelector('#r-close').onclick = () => modal.classList.add('hidden');
    modal.querySelectorAll('.r-match').forEach(btn => {
        btn.onclick = async () => {
            try {
                const meta = await api.receiptMeta(btn.dataset.id);
                if (!viewIsCurrent(state.tok)) return;
                openReceiptMatchModal(meta);
            } catch (e) { alert(`open failed: ${e.message}`); }
        };
    });
}
