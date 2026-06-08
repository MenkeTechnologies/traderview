import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { currencyOptions } from '../_currencies.js';

function renderInventoryChart(templates, filters) {
    const el = document.getElementById('set-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const tradeTpl   = (templates || []).filter(x => x.scope === 'trade').length;
    const journalTpl = (templates || []).filter(x => x.scope === 'journal').length;
    const defaultTpl = (templates || []).filter(x => x.is_default).length;
    const filterCt   = (filters || []).length;
    const defaultFi  = (filters || []).filter(x => x.is_default).length;
    const labels = [
        t('view.settings.chart.trade_tpl'),
        t('view.settings.chart.journal_tpl'),
        t('view.settings.chart.default_tpl'),
        t('view.settings.chart.filters'),
        t('view.settings.chart.default_filters'),
    ];
    const ys = [tradeTpl, journalTpl, defaultTpl, filterCt, defaultFi];
    if (ys.reduce((a, b) => a + b, 0) < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.settings.empty_chart">${esc(t('view.settings.empty_chart'))}</div>`;
        return;
    }
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 180,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.settings.chart.kind') },
            { label: t('view.settings.chart.count'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTemplateAgeChart(templates) {
    const el = document.getElementById('set-age-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const now = Date.now();
    const rows = (templates || [])
        .filter(tpl => tpl && tpl.updated_at)
        .map(tpl => ({
            name: tpl.name || '?',
            age: (now - new Date(tpl.updated_at).getTime()) / (1000 * 60 * 60 * 24),
        }))
        .filter(r => Number.isFinite(r.age))
        .sort((a, b) => b.age - a.age)
        .slice(0, 30);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.settings.empty_age_chart">${esc(t('view.settings.empty_age_chart'))}</div>`;
        return;
    }
    const labels = rows.map(r => r.name);
    const ys = rows.map(r => r.age);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.settings.chart.tpl_idx') },
            { label: t('view.settings.chart.age_days'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + 'd') },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

export async function renderSettings(mount, state) {
    const tok = currentViewToken();
    const [s, filters, templates, ds] = await Promise.all([
        api.settings(),
        api.listFilters(),
        api.noteTemplates(),
        api.dataSources(),
    ]);
    if (!viewIsCurrent(tok)) return;
    const accountOptions = state.accounts.map(a =>
        `<option value="${a.id}" ${a.id === s.default_account_id ? 'selected' : ''}>${esc(a.broker)} · ${esc(a.name)}</option>`
    ).join('');
    mount.innerHTML = `
        <h1 data-i18n="view.settings.h1.settings" class="view-title">// SETTINGS</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.appearance">Appearance</h2>
            <p data-i18n="view.settings.hint.crt_scanlines_neon_border_pulse_and_dark_light_the" class="muted small">CRT scanlines, neon-border pulse, and dark/light theme are toggled from the buttons in the topbar. Color scheme switches the whole HUD palette — picks below.</p>
            <div class="settings-scheme">
                <div class="scheme-grid" id="hudSchemeGrid"></div>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.profile">Profile</h2>
            <form id="settings-form" class="inline-form">
                <label><span data-i18n="view.settings.label.default_account">Default account</span>
                    <select name="default_account_id">
                        <option data-i18n="view.settings.opt.none" value="">(none)</option>${accountOptions}
                    </select>
                </label>
                <label><span data-i18n="view.settings.label.base_currency">Base currency</span>
                    <select name="base_currency">${currencyOptions(s.base_currency)}</select></label>
                <label><span data-i18n="view.settings.label.timezone">Timezone</span>
                    <input name="timezone" value="${esc(s.timezone)}"></label>
                <label><span data-i18n="view.settings.label.theme">Theme</span>
                    <select name="theme">
                        <option data-i18n="view.settings.opt.cyberpunk" value="cyberpunk" ${s.theme === 'cyberpunk' ? 'selected' : ''}>Cyberpunk</option>
                        <option data-i18n="view.settings.opt.dark" value="dark" ${s.theme === 'dark' ? 'selected' : ''}>Dark</option>
                    </select>
                </label>
                <label><span data-i18n="view.settings.label.starting_cash">Starting cash</span>
                    <input type="number" step="0.01" name="starting_cash" value="${s.starting_cash}"></label>
                <label><span data-i18n="view.settings.label.commission_per_share">Commission / share</span>
                    <input type="number" step="0.01" name="commission_per_share" value="${s.commission_per_share}">
                </label>
                <label><span data-i18n="view.settings.label.commission_per_contract">Commission / contract</span>
                    <input type="number" step="0.01" name="commission_per_contract" value="${s.commission_per_contract}">
                </label>
                <label><span data-i18n="view.settings.label.auto_flatten">Auto-flatten (new trade after going flat)</span>
                    <input type="checkbox" name="auto_flatten" ${s.auto_flatten ? 'checked' : ''}>
                </label>
                <label><span data-i18n="view.settings.label.require_account_tag">Always require account tag on import</span>
                    <input type="checkbox" name="require_account_tag" ${s.require_account_tag ? 'checked' : ''}>
                </label>
                <button data-i18n="view.settings.btn.save" class="primary" type="submit">Save</button>
            </form>
            <p data-i18n="view.settings.hint.commission_rates_fill_in_only_when_the_broker_file" class="muted small">
                Commission rates fill in only when the broker file omits fees (fee = 0).
                Mirrors TraderVue's "manual rate" behavior — won't double-count.
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.data_sources">Data Sources</h2>
            <p data-i18n="view.settings.hint.data_sources" class="muted small">
                API keys for market-data providers. Saved per-user in the database;
                masked as <code>***</code> after save so the secret is never re-displayed.
                Leave a field as <code>***</code> to keep the existing value when saving.
                Empty a field to clear it.
            </p>
            <form id="data-sources-form" class="inline-form">
                <label><span data-i18n="view.settings.label.finnhub_api_key">Finnhub API key</span>
                    <input type="password" name="finnhub_api_key" autocomplete="off" data-secret-field
                           value="${esc(ds.finnhub_api_key || '')}"
                           placeholder="finnhub.io key — used by REST + WS live ticks"
                           data-i18n-placeholder="view.settings.placeholder.finnhub_api_key"
                           style="min-width:280px">
                </label>
                <label><span data-i18n="view.settings.label.alpaca_key_id">Alpaca key ID</span>
                    <input type="text" name="alpaca_key_id" autocomplete="off" data-secret-field
                           value="${esc(ds.alpaca_key_id || '')}"
                           placeholder="APCA-API-KEY-ID"
                           data-i18n-placeholder="view.settings.placeholder.alpaca_key_id">
                </label>
                <label><span data-i18n="view.settings.label.alpaca_secret_key">Alpaca secret</span>
                    <input type="password" name="alpaca_secret_key" autocomplete="off" data-secret-field
                           value="${esc(ds.alpaca_secret_key || '')}"
                           placeholder="APCA-API-SECRET-KEY"
                           data-i18n-placeholder="view.settings.placeholder.alpaca_secret_key">
                </label>
                <label style="flex-direction:row;align-items:center;gap:6px">
                    <input type="checkbox" name="alpaca_paper" ${ds.alpaca_paper ? 'checked' : ''}>
                    <span data-i18n="view.settings.label.alpaca_paper">Alpaca paper-trading mode</span>
                </label>
                <hr style="flex:1 1 100%;border:0;border-top:1px solid var(--border);margin:6px 0">
                <p data-i18n="view.settings.hint.sip_sources" class="muted small" style="flex:1 1 100%">
                    SIP (consolidated tape) feeds — CTA + UTP for stocks, OPRA for options.
                    Required for real-time scalping; free IEX-only / Finnhub paths are sufficient
                    for charting + delayed quotes.
                </p>
                <label><span data-i18n="view.settings.label.polygon_api_key">Polygon.io API key</span>
                    <input type="password" name="polygon_api_key" autocomplete="off" data-secret-field
                           value="${esc(ds.polygon_api_key || '')}"
                           placeholder="polygon.io key — Advanced tier for full SIP tape"
                           data-i18n-placeholder="view.settings.placeholder.polygon_api_key"
                           style="min-width:280px">
                </label>
                <label><span data-i18n="view.settings.label.databento_api_key">Databento API key</span>
                    <input type="password" name="databento_api_key" autocomplete="off" data-secret-field
                           value="${esc(ds.databento_api_key || '')}"
                           placeholder="databento.com key — direct CTA / UTP / OPRA feeds"
                           data-i18n-placeholder="view.settings.placeholder.databento_api_key"
                           style="min-width:280px">
                </label>
                <label style="flex-direction:row;align-items:center;gap:6px">
                    <input type="checkbox" name="alpaca_use_sip_feed" ${ds.alpaca_use_sip_feed ? 'checked' : ''}>
                    <span data-i18n="view.settings.label.alpaca_use_sip_feed">Use Alpaca SIP feed (Algo Trader Plus required)</span>
                </label>
                <hr style="flex:1 1 100%;border:0;border-top:1px solid var(--border);margin:6px 0">
                <p data-i18n="view.settings.hint.tradier" class="muted small" style="flex:1 1 100%">
                    Tradier brokerage credentials — required when an algo strategy's account is on Tradier. Sandbox = sandbox.tradier.com (paper); off = api.tradier.com (live).
                </p>
                <label><span data-i18n="view.settings.label.tradier_access_token">Tradier access token</span>
                    <input type="password" name="tradier_access_token" autocomplete="off" data-secret-field
                           value="${esc(ds.tradier_access_token || '')}"
                           placeholder="Tradier Bearer token (Settings → API Access on tradier.com)"
                           data-i18n-placeholder="view.settings.placeholder.tradier_access_token"
                           style="min-width:280px">
                </label>
                <label><span data-i18n="view.settings.label.tradier_account_id">Tradier account ID</span>
                    <input type="text" name="tradier_account_id" autocomplete="off" data-secret-field
                           value="${esc(ds.tradier_account_id || '')}"
                           placeholder="Account number from Tradier dashboard (e.g. 6YA00001)"
                           data-i18n-placeholder="view.settings.placeholder.tradier_account_id">
                </label>
                <label style="flex-direction:row;align-items:center;gap:6px">
                    <input type="checkbox" name="tradier_sandbox" ${ds.tradier_sandbox !== false ? 'checked' : ''}>
                    <span data-i18n="view.settings.label.tradier_sandbox">Use Tradier sandbox (recommended)</span>
                </label>
                <button data-i18n="view.settings.btn.save_data_sources" class="primary" type="submit">Save data sources</button>
                <button data-i18n="view.settings.btn.test_finnhub" class="btn btn-secondary" type="button" id="ds-test-finnhub">Test Finnhub</button>
                <button data-i18n="view.settings.btn.test_alpaca" class="btn btn-secondary" type="button" id="ds-test-alpaca">Test Alpaca</button>
                <button data-i18n="view.settings.btn.test_tradier" class="btn btn-secondary" type="button" id="ds-test-tradier">Test Tradier</button>
                <hr style="flex:1 1 100%;border:0;border-top:1px solid var(--border);margin:6px 0">
                <p data-i18n="view.settings.hint.tastytrade" class="muted small" style="flex:1 1 100%">
                    Tastytrade brokerage credentials — required when an algo strategy's account is on Tastytrade. Either save a long-lived session token OR username + password (the backend mints a token via POST /sessions on demand).
                </p>
                <label><span data-i18n="view.settings.label.tastytrade_login">Tastytrade login (email or username)</span>
                    <input type="text" name="tastytrade_login" autocomplete="off" data-secret-field
                           value="${esc(ds.tastytrade_login || '')}"
                           placeholder="user@example.com"
                           data-i18n-placeholder="view.settings.placeholder.tastytrade_login">
                </label>
                <label><span data-i18n="view.settings.label.tastytrade_password">Tastytrade password</span>
                    <input type="password" name="tastytrade_password" autocomplete="off" data-secret-field
                           value="${esc(ds.tastytrade_password || '')}"
                           placeholder="Account password"
                           data-i18n-placeholder="view.settings.placeholder.tastytrade_password">
                </label>
                <label><span data-i18n="view.settings.label.tastytrade_session_token">Tastytrade session token (optional)</span>
                    <input type="password" name="tastytrade_session_token" autocomplete="off" data-secret-field
                           value="${esc(ds.tastytrade_session_token || '')}"
                           placeholder="Long-lived session token from remember-me login"
                           data-i18n-placeholder="view.settings.placeholder.tastytrade_session_token"
                           style="min-width:280px">
                </label>
                <label><span data-i18n="view.settings.label.tastytrade_account_number">Tastytrade account number</span>
                    <input type="text" name="tastytrade_account_number" autocomplete="off" data-secret-field
                           value="${esc(ds.tastytrade_account_number || '')}"
                           placeholder="Account # from Tastytrade dashboard (e.g. 5WX12345)"
                           data-i18n-placeholder="view.settings.placeholder.tastytrade_account_number">
                </label>
                <label style="flex-direction:row;align-items:center;gap:6px">
                    <input type="checkbox" name="tastytrade_sandbox" ${ds.tastytrade_sandbox !== false ? 'checked' : ''}>
                    <span data-i18n="view.settings.label.tastytrade_sandbox">Use Tastytrade sandbox (api.cert.tastyworks.com)</span>
                </label>
                <button data-i18n="view.settings.btn.test_tastytrade" class="btn btn-secondary" type="button" id="ds-test-tastytrade">Test Tastytrade</button>
                <p data-i18n="view.settings.hint.ibkr" class="muted small" style="flex:1 1 100%">
                    Interactive Brokers Client Portal Web API credentials. Run IB Gateway or Client Portal Gateway locally and log in via web — leave Bearer token empty for cookie-jar auth. For cloud / OAuth setups paste the Bearer token. Base URL defaults to https://localhost:5000/v1/api.
                </p>
                <label><span data-i18n="view.settings.label.ibkr_account_id">IBKR account ID</span>
                    <input type="text" name="ibkr_account_id" autocomplete="off" data-secret-field
                           value="${esc(ds.ibkr_account_id || '')}"
                           placeholder="DU1234567 (paper) or U1234567 (live)"
                           data-i18n-placeholder="view.settings.placeholder.ibkr_account_id">
                </label>
                <label><span data-i18n="view.settings.label.ibkr_base_url">IBKR base URL (optional)</span>
                    <input type="text" name="ibkr_base_url" autocomplete="off" data-secret-field
                           value="${esc(ds.ibkr_base_url || '')}"
                           placeholder="https://localhost:5000/v1/api"
                           data-i18n-placeholder="view.settings.placeholder.ibkr_base_url">
                </label>
                <label><span data-i18n="view.settings.label.ibkr_bearer_token">IBKR Bearer token (optional, OAuth)</span>
                    <input type="password" name="ibkr_bearer_token" autocomplete="off" data-secret-field
                           value="${esc(ds.ibkr_bearer_token || '')}"
                           placeholder="leave empty to use local gateway cookie auth"
                           data-i18n-placeholder="view.settings.placeholder.ibkr_bearer_token">
                </label>
                <button data-i18n="view.settings.btn.test_ibkr" class="btn btn-secondary" type="button" id="ds-test-ibkr">Test IBKR</button>
                <span id="ds-test-out" class="muted small" style="margin-left:8px"></span>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.notes_templates">Notes Templates</h2>
            <table class="trades">
                <thead><tr><th data-i18n="view.settings.th.name">Name</th><th data-i18n="view.settings.th.scope">Scope</th><th data-i18n="view.settings.th.default">Default</th><th data-i18n="view.settings.th.updated">Updated</th><th></th></tr></thead>
                <tbody>${templates.map(tpl => `
                    <tr><td>${esc(tpl.name)}</td>
                    <td>${esc(tpl.scope)}</td>
                    <td>${tpl.is_default ? '✓' : ''}</td>
                    <td>${fmtDateTime(tpl.updated_at)}</td>
                    <td>
                        <button data-i18n="view.settings.btn.edit" class="link" data-edit-tpl='${esc(JSON.stringify(tpl))}'>edit</button>
                        <button data-i18n="view.settings.btn.delete" class="link" data-del-tpl="${tpl.id}">delete</button>
                    </td></tr>
                `).join('') || `<tr><td colspan="5" class="muted">${esc(t('view.settings.empty.templates'))}</td></tr>`}
                </tbody>
            </table>
            <form id="tpl-form" class="inline-form" style="margin-top:10px">
                <input name="name" placeholder="template name" data-i18n-placeholder="view.settings.placeholder.template_name" required>
                <select name="scope">
                    <option data-i18n="view.settings.opt.trade" value="trade">trade</option>
                    <option data-i18n="view.settings.opt.journal" value="journal">journal</option>
                </select>
                <label style="flex-direction:row;align-items:center;gap:6px">
                    <input type="checkbox" name="is_default"> default
                </label>
                <textarea name="body_md" placeholder="markdown body — used as default when creating notes for the selected scope" data-i18n-placeholder="view.settings.placeholder.template_body" rows="4" style="flex:1 1 100%"></textarea>
                <button data-i18n="view.settings.btn.save_template" class="primary" type="submit">Save template</button>
            </form>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.saved_filter_sets">Saved filter sets</h2>
            ${filters.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.settings.th.name_2">Name</th><th data-i18n="view.settings.th.default_2">Default</th><th data-i18n="view.settings.th.created">Created</th><th></th></tr></thead>
                <tbody>${filters.map(f => `
                    <tr><td>${esc(f.name)}</td><td>${f.is_default ? '✓' : ''}</td>
                    <td>${fmtDateTime(f.created_at)}</td>
                    <td><button data-i18n="view.settings.btn.delete_2" class="link" data-del-f="${f.id}">delete</button></td></tr>
                `).join('')}</tbody></table>` : '<p data-i18n="view.settings.hint.no_saved_filters" class="muted">No saved filters.</p>'}
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.your_user_id">Your user ID</h2>
            <p data-i18n="view.settings.hint.share_this_with_someone_if_they_want_to_mentor_you">Share this with someone if they want to mentor you.</p>
            <code>${esc(state.me?.id || '')}</code>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.inventory_chart">Inventory: templates &amp; filters</h2>
            <div id="set-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.settings.h2.tpl_age_chart">Template age (days since last update)</h2>
            <div id="set-age-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.settings.hint.tpl_age" class="muted small">Days since each template's last <code>updated_at</code> timestamp. Reveals which templates are actively maintained vs stale. Orthogonal to the inventory-count chart above.</p>
        </div>
    `;

    renderInventoryChart(templates, filters);
    renderTemplateAgeChart(templates);

    // Repaint the color-scheme grid into the Appearance panel.
    if (window.tvHud && typeof window.tvHud.remountSchemeGrid === 'function') {
        window.tvHud.remountSchemeGrid();
    }

    mount.querySelector('#settings-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = Object.assign({}, s, {
            default_account_id: fd.get('default_account_id') || null,
            base_currency: fd.get('base_currency'),
            timezone: fd.get('timezone'),
            theme: fd.get('theme'),
            starting_cash: Number(fd.get('starting_cash')),
            commission_per_share: Number(fd.get('commission_per_share') || 0),
            commission_per_contract: Number(fd.get('commission_per_contract') || 0),
            auto_flatten: !!fd.get('auto_flatten'),
            require_account_tag: !!fd.get('require_account_tag'),
        });
        await api.updateSettings(body);
        if (!viewIsCurrent(tok)) return;
        renderSettings(mount, state);
    });

    // Attach a reveal toggle (👁 ↔ 🙈) to every secret field. Click
    // once: fetch the unmasked value and switch the input to type
    // "text" so the user can read it. Click again: re-mask and revert
    // to the masked placeholder so the next page reload is clean.
    // Caches the unmasked DTO on the mount so toggling multiple
    // fields doesn't refetch.
    let _revealCache = null;
    const fetchUnmasked = async () => {
        if (_revealCache) return _revealCache;
        _revealCache = await api.dataSourcesReveal();
        return _revealCache;
    };
    mount.querySelectorAll('input[data-secret-field]').forEach((input) => {
        const wrap = input.parentElement;
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'btn-reveal';
        btn.textContent = '👁';
        btn.title = t('view.settings.reveal_secret');
        btn.style.marginLeft = '4px';
        const wasPassword = input.type === 'password';
        let revealed = false;
        btn.addEventListener('click', async () => {
            if (revealed) {
                // Re-mask: revert to password type and restore placeholder.
                if (wasPassword) input.type = 'password';
                input.value = '***';
                revealed = false;
                btn.textContent = '👁';
                return;
            }
            btn.disabled = true;
            try {
                const dto = await fetchUnmasked();
                const v = dto[input.name];
                if (v == null || v === '') {
                    showToast(t('view.settings.reveal_empty', { field: input.name }), { level: 'info' });
                    return;
                }
                input.type = 'text';
                input.value = v;
                revealed = true;
                btn.textContent = '🙈';
            } catch (err) {
                showToast(t('view.settings.reveal_failed', { err: err.message || err }), { level: 'error' });
            } finally {
                btn.disabled = false;
            }
        });
        wrap.appendChild(btn);
    });

    mount.querySelector('#data-sources-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            await api.updateDataSources({
                finnhub_api_key:       fd.get('finnhub_api_key')       ?? null,
                alpaca_key_id:         fd.get('alpaca_key_id')         ?? null,
                alpaca_secret_key:     fd.get('alpaca_secret_key')     ?? null,
                alpaca_paper:          !!fd.get('alpaca_paper'),
                polygon_api_key:       fd.get('polygon_api_key')       ?? null,
                databento_api_key:     fd.get('databento_api_key')     ?? null,
                alpaca_use_sip_feed:   !!fd.get('alpaca_use_sip_feed'),
                tradier_access_token:      fd.get('tradier_access_token')      ?? null,
                tradier_account_id:        fd.get('tradier_account_id')        ?? null,
                tradier_sandbox:           !!fd.get('tradier_sandbox'),
                tastytrade_login:          fd.get('tastytrade_login')           ?? null,
                tastytrade_password:       fd.get('tastytrade_password')        ?? null,
                tastytrade_session_token:  fd.get('tastytrade_session_token')   ?? null,
                tastytrade_account_number: fd.get('tastytrade_account_number')  ?? null,
                tastytrade_sandbox:        !!fd.get('tastytrade_sandbox'),
                ibkr_account_id:           fd.get('ibkr_account_id')           ?? null,
                ibkr_base_url:             fd.get('ibkr_base_url')             ?? null,
                ibkr_bearer_token:         fd.get('ibkr_bearer_token')         ?? null,
            });
            showToast(t('view.settings.toast.data_sources_saved'), { level: 'success' });
            if (!viewIsCurrent(tok)) return;
            renderSettings(mount, state);
        } catch (err) {
            showToast(t('view.settings.toast.data_sources_save_failed', { msg: err.message || err }), { level: 'error' });
        }
    });

    // Test Finnhub — REST probe (no WS) against the quote endpoint.
    // Falls back to stored key when the field still shows `***`.
    mount.querySelector('#ds-test-finnhub').addEventListener('click', async () => {
        const form = mount.querySelector('#data-sources-form');
        const out = mount.querySelector('#ds-test-out');
        const btn = mount.querySelector('#ds-test-finnhub');
        const mask = '***';
        const fd = new FormData(form);
        const liveValue = (k) => {
            const v = String(fd.get(k) || '').trim();
            return (!v || v === mask) ? null : v;
        };
        btn.disabled = true;
        out.textContent = t('view.settings.test_finnhub.connecting');
        out.style.color = '';
        try {
            const r = await api.testFinnhub({
                api_key: liveValue('finnhub_api_key'),
            });
            if (r.ok) {
                out.textContent = t('view.settings.test_finnhub.ok');
                out.style.color = '#39ff14';
                showToast(t('view.settings.test_finnhub.toast_ok'), { level: 'success' });
            } else {
                const msg = r.detail?.msg || JSON.stringify(r.detail || {});
                out.textContent = t('view.settings.test_finnhub.fail', { msg });
                out.style.color = '#ff5a5a';
                showToast(t('view.settings.test_finnhub.toast_fail', { msg }), { level: 'error' });
            }
        } catch (err) {
            const msg = err?.message || String(err);
            out.textContent = t('view.settings.test_finnhub.fail', { msg });
            out.style.color = '#ff5a5a';
            showToast(t('view.settings.test_finnhub.toast_fail', { msg }), { level: 'error' });
        } finally {
            btn.disabled = false;
        }
    });

    // Test Alpaca — opens a WS via the backend with the CURRENT form
    // values (lets the user verify a fresh key without saving first).
    // Sends `key_id: null` / `secret: null` if the field still holds
    // the masked "***" placeholder, so the backend falls back to the
    // stored creds.
    mount.querySelector('#ds-test-alpaca').addEventListener('click', async () => {
        const form = mount.querySelector('#data-sources-form');
        const out = mount.querySelector('#ds-test-out');
        const btn = mount.querySelector('#ds-test-alpaca');
        const mask = '***';
        const fd = new FormData(form);
        const liveValue = (k) => {
            const v = String(fd.get(k) || '').trim();
            return (!v || v === mask) ? null : v;
        };
        btn.disabled = true;
        out.textContent = t('view.settings.test_alpaca.connecting');
        out.style.color = '';
        try {
            const r = await api.testAlpaca({
                key_id:  liveValue('alpaca_key_id'),
                secret:  liveValue('alpaca_secret_key'),
                use_sip: !!fd.get('alpaca_use_sip_feed'),
            });
            if (r.ok) {
                out.textContent = t('view.settings.test_alpaca.ok', { feed: r.feed.toUpperCase() });
                out.style.color = '#39ff14';
                showToast(t('view.settings.test_alpaca.toast_ok', { feed: r.feed.toUpperCase() }), { level: 'success' });
            } else {
                const msg = r.detail?.msg || JSON.stringify(r.detail || {});
                out.textContent = t('view.settings.test_alpaca.fail', { msg });
                out.style.color = '#ff5a5a';
                showToast(t('view.settings.test_alpaca.toast_fail', { msg }), { level: 'error' });
            }
        } catch (err) {
            const msg = err?.message || String(err);
            out.textContent = t('view.settings.test_alpaca.fail', { msg });
            out.style.color = '#ff5a5a';
            showToast(t('view.settings.test_alpaca.toast_fail', { msg }), { level: 'error' });
        } finally {
            btn.disabled = false;
        }
    });

    // Test Tradier — REST probe against /balances. Falls back to stored
    // creds when fields still show `***`.
    mount.querySelector('#ds-test-tradier').addEventListener('click', async () => {
        const form = mount.querySelector('#data-sources-form');
        const out = mount.querySelector('#ds-test-out');
        const btn = mount.querySelector('#ds-test-tradier');
        const mask = '***';
        const fd = new FormData(form);
        const liveValue = (k) => {
            const v = String(fd.get(k) || '').trim();
            return (!v || v === mask) ? null : v;
        };
        btn.disabled = true;
        out.textContent = t('view.settings.test_tradier.connecting');
        out.style.color = '';
        try {
            const r = await api.testTradier({
                access_token: liveValue('tradier_access_token'),
                account_id:   liveValue('tradier_account_id'),
                sandbox:      !!fd.get('tradier_sandbox'),
            });
            if (r.ok) {
                const env = r.detail?.sandbox ? 'SANDBOX' : 'LIVE';
                out.textContent = t('view.settings.test_tradier.ok', { env });
                out.style.color = '#39ff14';
                showToast(t('view.settings.test_tradier.toast_ok', { env }), { level: 'success' });
            } else {
                const msg = r.detail?.msg || JSON.stringify(r.detail || {});
                out.textContent = t('view.settings.test_tradier.fail', { msg });
                out.style.color = '#ff5a5a';
                showToast(t('view.settings.test_tradier.toast_fail', { msg }), { level: 'error' });
            }
        } catch (err) {
            const msg = err?.message || String(err);
            out.textContent = t('view.settings.test_tradier.fail', { msg });
            out.style.color = '#ff5a5a';
            showToast(t('view.settings.test_tradier.toast_fail', { msg }), { level: 'error' });
        } finally {
            btn.disabled = false;
        }
    });

    // Test Tastytrade — same shape as Tradier probe but supports
    // session-token OR login/password auth.
    mount.querySelector('#ds-test-tastytrade').addEventListener('click', async () => {
        const form = mount.querySelector('#data-sources-form');
        const out = mount.querySelector('#ds-test-out');
        const btn = mount.querySelector('#ds-test-tastytrade');
        const mask = '***';
        const fd = new FormData(form);
        const liveValue = (k) => {
            const v = String(fd.get(k) || '').trim();
            return (!v || v === mask) ? null : v;
        };
        btn.disabled = true;
        out.textContent = t('view.settings.test_tastytrade.connecting');
        out.style.color = '';
        try {
            const r = await api.testTastytrade({
                login:          liveValue('tastytrade_login'),
                password:       liveValue('tastytrade_password'),
                session_token:  liveValue('tastytrade_session_token'),
                account_number: liveValue('tastytrade_account_number'),
                sandbox:        !!fd.get('tastytrade_sandbox'),
            });
            if (r.ok) {
                const env = r.detail?.sandbox ? 'SANDBOX' : 'LIVE';
                out.textContent = t('view.settings.test_tastytrade.ok', { env });
                out.style.color = '#39ff14';
                showToast(t('view.settings.test_tastytrade.toast_ok', { env }), { level: 'success' });
            } else {
                const msg = r.detail?.msg || JSON.stringify(r.detail || {});
                out.textContent = t('view.settings.test_tastytrade.fail', { msg });
                out.style.color = '#ff5a5a';
                showToast(t('view.settings.test_tastytrade.toast_fail', { msg }), { level: 'error' });
            }
        } catch (err) {
            const msg = err?.message || String(err);
            out.textContent = t('view.settings.test_tastytrade.fail', { msg });
            out.style.color = '#ff5a5a';
            showToast(t('view.settings.test_tastytrade.toast_fail', { msg }), { level: 'error' });
        } finally {
            btn.disabled = false;
        }
    });

    // Test IBKR — probes the Client Portal Web API /portfolio/{id}/summary
    // endpoint. Falls back to stored creds when fields are masked.
    mount.querySelector('#ds-test-ibkr').addEventListener('click', async () => {
        const form = mount.querySelector('#data-sources-form');
        const out = mount.querySelector('#ds-test-out');
        const btn = mount.querySelector('#ds-test-ibkr');
        const mask = '***';
        const fd = new FormData(form);
        const liveValue = (k) => {
            const v = String(fd.get(k) || '').trim();
            return (!v || v === mask) ? null : v;
        };
        btn.disabled = true;
        out.textContent = t('view.settings.test_ibkr.connecting');
        out.style.color = '';
        try {
            const r = await api.testIbkr({
                account_id:   liveValue('ibkr_account_id'),
                base_url:     liveValue('ibkr_base_url'),
                bearer_token: liveValue('ibkr_bearer_token'),
            });
            if (r.ok) {
                const mode = r.detail?.auth_mode || 'cookie_jar';
                out.textContent = t('view.settings.test_ibkr.ok', { mode });
                out.style.color = '#39ff14';
                showToast(t('view.settings.test_ibkr.toast_ok', { mode }), { level: 'success' });
            } else {
                const msg = r.detail?.msg || JSON.stringify(r.detail || {});
                out.textContent = t('view.settings.test_ibkr.fail', { msg });
                out.style.color = '#ff5a5a';
                showToast(t('view.settings.test_ibkr.toast_fail', { msg }), { level: 'error' });
            }
        } catch (err) {
            const msg = err?.message || String(err);
            out.textContent = t('view.settings.test_ibkr.fail', { msg });
            out.style.color = '#ff5a5a';
            showToast(t('view.settings.test_ibkr.toast_fail', { msg }), { level: 'error' });
        } finally {
            btn.disabled = false;
        }
    });

    mount.querySelector('#tpl-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        await api.upsertNoteTemplate(
            fd.get('name'),
            fd.get('scope'),
            fd.get('body_md') || '',
            !!fd.get('is_default'),
        );
        if (!viewIsCurrent(tok)) return;
        renderSettings(mount, state);
    });
    mount.querySelectorAll('[data-edit-tpl]').forEach(b =>
        b.addEventListener('click', () => {
            const t = JSON.parse(b.dataset.editTpl);
            const f = mount.querySelector('#tpl-form');
            if (!f) return;
            f.name.value = t.name;
            f.scope.value = t.scope;
            f.body_md.value = t.body_md;
            f.is_default.checked = t.is_default;
        }));
    mount.querySelectorAll('[data-del-tpl]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteNoteTemplate(b.dataset.delTpl);
            if (!viewIsCurrent(tok)) return;
            renderSettings(mount, state);
        }));
    mount.querySelectorAll('[data-del-f]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteFilter(b.dataset.delF);
            if (!viewIsCurrent(tok)) return;
            renderSettings(mount, state);
        }));
}
