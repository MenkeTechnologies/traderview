// Multi-field setup wizard modal for creating a new broker, business,
// or account entity. Replaces `window.prompt()` — which silently
// returns null in the Tauri WebView — with a real form rendered
// into #tv-dialog-root.
//
// Public:
//   await openSetupWizard({ kind: 'broker' | 'business' | 'account' })
//     → resolves to the created entity (server response), or null if
//       the user cancelled / the API call failed.

import { api } from './api.js';
import { t, applyUiI18n } from './i18n.js';
import { esc } from './util.js';
import { showToast } from './toast.js';
import { currencyOptions } from './_currencies.js';

function slugify(s) {
    return String(s || '')
        .trim().toLowerCase()
        .replace(/[^a-z0-9]+/g, '_')
        .replace(/^_+|_+$/g, '');
}

// Brokers TraderView supports via either dedicated CSV import parsers OR
// the algo trading dispatcher (REST + WS adapter modules). Slugs MUST
// stay in sync with two consumers:
//   1. CSV import — `crates/traderview-import/src/brokers.rs::*Parser::source()`.
//      Unknown slugs fall through to the generic parser (still works,
//      just less broker-specific column mapping).
//   2. Algo dispatcher — `crates/traderview-web/src/routes/algo.rs::
//      ALGO_SUPPORTED_BROKERS` + `crates/traderview-db/src/broker_dispatcher.rs`.
//      Algo-eligible slugs MUST match exactly or strategy creation
//      rejects the account.
//
// Algo-eligible brokers (REST submit + WS fill pump wired):
//   alpaca, tradier, ibkr, schwab/td, tastytrade.
const SUPPORTED_BROKERS = [
    { slug: 'alpaca',       label: 'Alpaca',              home: 'https://alpaca.markets' },
    { slug: 'tradier',      label: 'Tradier',             home: 'https://tradier.com' },
    { slug: 'tastytrade',   label: 'tastytrade',          home: 'https://tastytrade.com' },
    { slug: 'ibkr',         label: 'Interactive Brokers', home: 'https://www.interactivebrokers.com' },
    { slug: 'schwab',       label: 'Charles Schwab',      home: 'https://www.schwab.com' },
    { slug: 'tdameritrade', label: 'TD Ameritrade',       home: 'https://www.tdameritrade.com' },
    { slug: 'webull',       label: 'Webull',              home: 'https://www.webull.com' },
    { slug: 'tos',          label: 'thinkorswim',         home: 'https://www.thinkorswim.com' },
    { slug: 'fidelity',     label: 'Fidelity',            home: 'https://www.fidelity.com' },
    { slug: 'etrade',       label: 'E*TRADE',             home: 'https://us.etrade.com' },
    { slug: 'tradestation', label: 'TradeStation',        home: 'https://www.tradestation.com' },
    { slug: 'lightspeed',   label: 'Lightspeed',          home: 'https://www.lightspeed.com' },
    { slug: 'das',          label: 'DAS Trader',          home: 'https://dastrader.com' },
    { slug: 'tradezero',    label: 'TradeZero',           home: 'https://www.tradezero.com' },
    { slug: 'robinhood',    label: 'Robinhood',           home: 'https://robinhood.com' },
];

function ensureMount() {
    let root = document.getElementById('tv-dialog-root');
    if (!root) {
        root = document.createElement('div');
        root.id = 'tv-dialog-root';
        document.body.appendChild(root);
    }
    return root;
}

function brokerFields() {
    const opts = SUPPORTED_BROKERS
        .map((b) => `<option value="${esc(b.slug)}" data-home="${esc(b.home)}" data-label="${esc(b.label)}">${esc(b.label)}</option>`)
        .join('');
    return `
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-broker-pick" data-i18n="wiz.broker.pick">${esc(t('wiz.broker.pick'))}</label>
            <select id="wiz-broker-pick" class="tv-dialog-input">
                ${opts}
                <option value="__custom__" data-i18n="wiz.broker.other">${esc(t('wiz.broker.other'))}</option>
            </select>
        </div>
        <div class="tv-wiz-row" id="wiz-custom-name-row" style="display:none">
            <label class="tv-wiz-label" for="wiz-name" data-i18n="wiz.broker.name">${esc(t('wiz.broker.name'))}</label>
            <input id="wiz-name" class="tv-dialog-input" type="text" autocomplete="off">
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-home" data-i18n="wiz.broker.home_url">${esc(t('wiz.broker.home_url'))}</label>
            <input id="wiz-home" class="tv-dialog-input" type="text" autocomplete="off"
                   placeholder="https://…">
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-notes" data-i18n="wiz.broker.notes">${esc(t('wiz.broker.notes'))}</label>
            <textarea id="wiz-notes" class="tv-dialog-input tv-wiz-textarea" rows="2"></textarea>
        </div>`;
}

function businessFields() {
    return `
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-name" data-i18n="wiz.biz.name">${esc(t('wiz.biz.name'))}</label>
            <input id="wiz-name" class="tv-dialog-input" type="text" autocomplete="off" required>
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-entity" data-i18n="wiz.biz.entity_type">${esc(t('wiz.biz.entity_type'))}</label>
            <select id="wiz-entity" class="tv-dialog-input">
                <option value="sole_prop" data-i18n="wiz.biz.et.sole_prop">${esc(t('wiz.biz.et.sole_prop'))}</option>
                <option value="llc" data-i18n="wiz.biz.et.llc">${esc(t('wiz.biz.et.llc'))}</option>
                <option value="s_corp" data-i18n="wiz.biz.et.s_corp">${esc(t('wiz.biz.et.s_corp'))}</option>
                <option value="c_corp" data-i18n="wiz.biz.et.c_corp">${esc(t('wiz.biz.et.c_corp'))}</option>
                <option value="partnership" data-i18n="wiz.biz.et.partnership">${esc(t('wiz.biz.et.partnership'))}</option>
            </select>
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-ein" data-i18n="wiz.biz.ein">${esc(t('wiz.biz.ein'))}</label>
            <input id="wiz-ein" class="tv-dialog-input" type="text" autocomplete="off"
                   placeholder="XX-XXXXXXX">
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-naics" data-i18n="wiz.biz.naics">${esc(t('wiz.biz.naics'))}</label>
            <input id="wiz-naics" class="tv-dialog-input" type="text" autocomplete="off"
                   placeholder="523900">
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-started" data-i18n="wiz.biz.started_at">${esc(t('wiz.biz.started_at'))}</label>
            <input id="wiz-started" class="tv-dialog-input" type="date" autocomplete="off">
        </div>`;
}

function collectBroker(root) {
    const pick = root.querySelector('#wiz-broker-pick');
    const home = root.querySelector('#wiz-home').value.trim();
    const notes = root.querySelector('#wiz-notes').value.trim();
    let displayName;
    let slug;
    if (pick.value === '__custom__') {
        const custom = root.querySelector('#wiz-name').value.trim();
        if (!custom) return null;
        displayName = custom;
        slug = slugify(custom);
    } else {
        slug = pick.value;
        displayName = pick.selectedOptions[0]?.dataset.label || pick.value;
    }
    const body = { display_name: displayName, slug };
    if (home) body.home_url = home;
    if (notes) body.notes = notes;
    return body;
}

function collectBusiness(root) {
    const name = root.querySelector('#wiz-name').value.trim();
    if (!name) return null;
    const entity = root.querySelector('#wiz-entity').value;
    const ein = root.querySelector('#wiz-ein').value.trim();
    const naics = root.querySelector('#wiz-naics').value.trim();
    const started = root.querySelector('#wiz-started').value;
    const body = { name, entity_type: entity || 'sole_prop' };
    if (ein) body.ein = ein;
    if (naics) body.naics_code = naics;
    if (started) body.started_at = started;
    return body;
}

function accountFields({ brokers, defaultBrokerSlug }) {
    // Pull from the live `brokers` table — the user's custom brokers
    // appear alongside the import-supported ones. Slugs from `brokers.slug`
    // are guaranteed to match parser dispatch by accounts::create's upsert.
    const opts = brokers
        .map((b) => `<option value="${esc(b.slug)}" data-label="${esc(b.display_name)}"
                ${b.slug === defaultBrokerSlug ? 'selected' : ''}>${esc(b.display_name)}</option>`)
        .join('');
    return `
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-acct-broker" data-i18n="wiz.acct.broker">${esc(t('wiz.acct.broker'))}</label>
            <select id="wiz-acct-broker" class="tv-dialog-input" ${brokers.length ? '' : 'disabled'}>
                ${opts || `<option disabled selected>${esc(t('wiz.acct.no_brokers'))}</option>`}
            </select>
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-name" data-i18n="wiz.acct.name">${esc(t('wiz.acct.name'))}</label>
            <input id="wiz-name" class="tv-dialog-input" type="text" autocomplete="off"
                   placeholder="${esc(t('wiz.acct.name_hint'))}" required>
        </div>
        <div class="tv-wiz-row">
            <label class="tv-wiz-label" for="wiz-ccy" data-i18n="wiz.acct.base_currency">${esc(t('wiz.acct.base_currency'))}</label>
            <select id="wiz-ccy" class="tv-dialog-input">${currencyOptions('USD')}</select>
        </div>`;
}

function collectAccount(root) {
    const brokerSel = root.querySelector('#wiz-acct-broker');
    const name = root.querySelector('#wiz-name').value.trim();
    if (!name || !brokerSel || !brokerSel.value) return null;
    return {
        broker: brokerSel.value,
        name,
        base_currency: root.querySelector('#wiz-ccy').value || 'USD',
    };
}

export async function openSetupWizard({ kind, defaultBrokerSlug } = {}) {
    const isBroker = kind === 'broker';
    const isAccount = kind === 'account';
    // Account wizard needs the live brokers list to pick the FK. Fetch
    // upfront so the form has the picker populated when it renders.
    let brokers = [];
    if (isAccount) {
        try { brokers = await api.brokersList(); } catch (e) {
            showToast(t('broker.list_failed', { err: e.message || String(e) }), { level: 'error' });
            return null;
        }
    }
    return new Promise((resolve) => {
        const root = ensureMount();
        const titleKey = isBroker ? 'wiz.broker.title'
            : isAccount ? 'wiz.acct.title' : 'wiz.biz.title';
        const subtitleKey = isBroker ? 'wiz.broker.subtitle'
            : isAccount ? 'wiz.acct.subtitle' : 'wiz.biz.subtitle';
        const fields = isBroker ? brokerFields()
            : isAccount ? accountFields({ brokers, defaultBrokerSlug })
            : businessFields();
        root.innerHTML = `
            <div class="tv-dialog-overlay" role="dialog" aria-modal="true">
                <div class="tv-dialog-card tv-dialog-info tv-wiz-card">
                    <div class="tv-dialog-title" data-i18n="${titleKey}">${esc(t(titleKey))}</div>
                    <div class="tv-dialog-message tv-wiz-subtitle" data-i18n="${subtitleKey}">${esc(t(subtitleKey))}</div>
                    <div class="tv-wiz-form">${fields}</div>
                    <div class="tv-dialog-actions">
                        <button type="button" class="tv-dialog-btn tv-dialog-cancel" data-i18n="dialog.btn.cancel">${esc(t('dialog.btn.cancel'))}</button>
                        <button type="button" class="tv-dialog-btn tv-dialog-confirm" data-i18n="wiz.btn.create">${esc(t('wiz.btn.create'))}</button>
                    </div>
                </div>
            </div>`;
        applyUiI18n(root);
        const overlay = root.querySelector('.tv-dialog-overlay');
        const nameInput = root.querySelector('#wiz-name');
        const confirmBtn = root.querySelector('.tv-dialog-confirm');
        const cancelBtn = root.querySelector('.tv-dialog-cancel');
        const brokerPick = root.querySelector('#wiz-broker-pick');
        const homeInput = root.querySelector('#wiz-home');
        const customRow = root.querySelector('#wiz-custom-name-row');

        // Broker picker → auto-fill the home URL and reveal the custom-name
        // row when "Other" is picked. Home URL only auto-fills when the user
        // hasn't typed anything there yet, so a manual edit isn't clobbered.
        let homeDirty = false;
        if (homeInput) homeInput.addEventListener('input', () => { homeDirty = true; });
        if (brokerPick) {
            const onPick = () => {
                const isCustom = brokerPick.value === '__custom__';
                if (customRow) customRow.style.display = isCustom ? '' : 'none';
                if (!homeDirty && homeInput) {
                    const home = brokerPick.selectedOptions[0]?.dataset.home || '';
                    homeInput.value = isCustom ? '' : home;
                }
                if (isCustom && nameInput) nameInput.focus();
            };
            brokerPick.addEventListener('change', onPick);
            onPick();
        }

        const close = (result) => {
            document.removeEventListener('keydown', onKey, true);
            root.innerHTML = '';
            resolve(result);
        };
        const cancel = () => close(null);
        const submit = async () => {
            const body = isBroker ? collectBroker(root)
                : isAccount ? collectAccount(root)
                : collectBusiness(root);
            if (!body) {
                // For broker "Other", the missing field is the custom
                // name input; for business it's the same #wiz-name input.
                const target = nameInput || root.querySelector('.tv-dialog-input');
                if (target) {
                    target.classList.add('tv-dialog-input-error');
                    target.focus();
                    setTimeout(() => target.classList.remove('tv-dialog-input-error'), 400);
                }
                return;
            }
            confirmBtn.disabled = true;
            try {
                const created = isBroker ? await api.brokerCreate(body)
                    : isAccount ? await api.createAccount(body.broker, body.name, body.base_currency)
                    : await api.businessCreate(body);
                close(created);
            } catch (e) {
                confirmBtn.disabled = false;
                const errKey = isBroker ? 'broker.create_failed'
                    : isAccount ? 'acct.create_failed' : 'biz.create_failed';
                showToast(t(errKey, { err: e.message || String(e) }), { level: 'error' });
            }
        };
        const onKey = (e) => {
            if (e.key === 'Escape') { e.preventDefault(); cancel(); }
            else if (e.key === 'Enter' && e.target.tagName !== 'TEXTAREA') {
                e.preventDefault(); submit();
            }
        };
        document.addEventListener('keydown', onKey, true);
        confirmBtn.addEventListener('click', submit);
        cancelBtn.addEventListener('click', cancel);
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) cancel();
        });
        requestAnimationFrame(() => {
            // Broker flow: focus the picker dropdown by default. Business
            // flow: focus the business name input.
            const initial = brokerPick || nameInput;
            initial?.focus();
        });
    });
}
