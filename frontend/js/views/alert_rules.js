// Alert Rules view — multi-rule, multi-type custom audio alerts with
// per-rule sound + TTS template editor. Generalization of the
// Squeeze Alerts view: any rule can fire any sound + any voice message
// on any subset of symbols.

import { esc } from '../util.js';
import * as engine from '../_alert_rules.js';
import { t as tr } from '../i18n.js';
import * as squeeze from '../_squeeze_alerts_inputs.js';   // reuse tick + adv parsers
import * as audio from '../_audio_alerts.js';

let state = engine.loadState();

export async function renderAlertRules(mount, _appState) {
    const caps = audio.audioCapabilities();
    mount.innerHTML = `
        <h1 data-i18n="view.alert_rules.h1.custom_alert_rules" class="view-title">// CUSTOM ALERT RULES</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.alert_rules.h2.audio_capability">Audio capability</h2>
            <div class="cards">
                ${card(tr('view.alert_rules.card.web_audio'),   caps.audio ? 'available' : 'NOT AVAILABLE', caps.audio ? 'pos' : 'neg')}
                ${card(tr('view.alert_rules.card.speech_tts'), caps.tts   ? 'available' : 'NOT AVAILABLE', caps.tts   ? 'pos' : 'neg')}
                ${card(tr('view.alert_rules.card.rules_saved'),  String(state.rules.length), state.rules.length ? 'pos' : '')}
                ${card(tr('view.alert_rules.card.enabled'),      String(state.rules.filter(r => r.enabled).length))}
            </div>
            <div class="inline-form" style="margin-top:8px">
                <button data-i18n="view.alert_rules.btn.bell" id="ar-test-bell"   class="secondary" type="button" ${!caps.audio ? 'disabled' : ''}>🔔 bell</button>
                <button data-i18n="view.alert_rules.btn.alarm" id="ar-test-alarm"  class="secondary" type="button" ${!caps.audio ? 'disabled' : ''}>🚨 alarm</button>
                <button data-i18n="view.alert_rules.btn.single_beep" id="ar-test-beep"   class="secondary" type="button" ${!caps.audio ? 'disabled' : ''}>· single beep</button>
                <button data-i18n="view.alert_rules.btn.double_beep" id="ar-test-double" class="secondary" type="button" ${!caps.audio ? 'disabled' : ''}>·· double beep</button>
                <button data-i18n="view.alert_rules.btn.tts" id="ar-test-tts"    class="secondary" type="button" ${!caps.tts   ? 'disabled' : ''}>🗣 TTS</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.alert_rules.h2.rules">Rules</h2>
            <div id="ar-rules"></div>
            <div class="inline-form" style="margin-top:10px">
                <label>+ Add rule of type
                    <select id="ar-new-type">
                        <option data-i18n="view.alert_rules.opt.squeeze" value="squeeze">squeeze</option>
                        <option data-i18n="view.alert_rules.opt.price_above" value="price_above">price above</option>
                        <option data-i18n="view.alert_rules.opt.price_below" value="price_below">price below</option>
                        <option data-i18n="view.alert_rules.opt.pct_change_in_window" value="pct_change">pct change in window</option>
                        <option data-i18n="view.alert_rules.opt.volume_spike_in_window" value="volume_spike">volume spike in window</option>
                    </select></label>
                <input id="ar-new-name" type="text" placeholder="rule name" data-i18n-placeholder="view.alert_rules.placeholder.name">
                <button data-i18n="view.alert_rules.btn.add" id="ar-add" class="primary" type="button">+ Add</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.alert_rules.h2.tick_feed">Tick feed</h2>
            <textarea id="ar-ticks" rows="5" placeholder="AAPL 1700000000 150.00 5000"></textarea>
            <textarea id="ar-adv" rows="2" placeholder="AAPL 50000000&#10;SMID 250000" style="margin-top:6px"></textarea>
            <div class="inline-form">
                <button data-i18n="view.alert_rules.btn.load_demo_aapl_smid_30_min_span" id="ar-demo" class="secondary" type="button">Load demo (AAPL + SMID, 30-min span)</button>
                <button data-i18n="view.alert_rules.btn.clear_feed" id="ar-clear-feed" class="secondary" type="button">Clear feed</button>
                <button data-i18n="view.alert_rules.btn.run_all_rules" id="ar-replay" class="primary" type="button">▶ Run all rules</button>
                <button data-i18n="view.alert_rules.btn.stop_tts" id="ar-stop-tts" class="secondary" type="button">⏹ Stop TTS</button>
            </div>
        </div>

        <div id="ar-errors" class="boot" style="display:none"></div>
        <div id="ar-events"></div>
    `;
    bindTestButtons();
    bindRulesPanel();
    bindAddRule();
    bindFeedButtons();
    renderRulesList();
}

function bindTestButtons() {
    document.getElementById('ar-test-bell').addEventListener('click',   () => audio.playSound('bell',         { volume: 0.3 }));
    document.getElementById('ar-test-alarm').addEventListener('click',  () => audio.playSound('alarm',        { volume: 0.3 }));
    document.getElementById('ar-test-beep').addEventListener('click',   () => audio.playSound('single_beep',  { volume: 0.3 }));
    document.getElementById('ar-test-double').addEventListener('click', () => audio.playSound('double_beep',  { volume: 0.3 }));
    document.getElementById('ar-test-tts').addEventListener('click',    () =>
        audio.speakAlert(tr('view.alert_rules.tts.test_phrase'), { volume: 0.6 }));
}

function bindRulesPanel() {
    document.getElementById('ar-rules').addEventListener('click', (ev) => {
        const t = ev.target;
        if (!(t instanceof HTMLElement)) return;
        const id = t.dataset.ruleId;
        if (!id) return;
        if (t.dataset.action === 'remove') {
            if (window.confirm(tr('view.alert_rules.confirm.remove'))) {
                state = engine.removeRule(state, id);
                engine.saveState(state);
                renderRulesList();
            }
        }
    });
    document.getElementById('ar-rules').addEventListener('change', (ev) => {
        const t = ev.target;
        if (!(t instanceof HTMLElement)) return;
        const id = t.dataset.ruleId;
        if (!id) return;
        const field = t.dataset.field;
        if (!field) return;
        const rule = state.rules.find(r => r.id === id);
        if (!rule) return;
        const patch = patchFromInput(t, field, rule);
        if (!patch) return;
        state = engine.updateRule(state, id, patch);
        engine.saveState(state);
    });
}

function patchFromInput(el, field, rule) {
    if (field === 'enabled') return { enabled: el.checked };
    if (field === 'name') return { name: el.value };
    if (field === 'sound') return { sound: el.value };
    if (field === 'tts_template') return { tts_template: el.value };
    if (field === 'cooldown_seconds') return { cooldown_seconds: Number(el.value) };
    if (field === 'watchlist') {
        const wl = String(el.value || '').split(/[\s,]+/).filter(Boolean).map(s => s.toUpperCase());
        return { watchlist: wl };
    }
    if (field.startsWith('params.')) {
        const key = field.slice('params.'.length);
        const params = { ...rule.params, [key]: Number(el.value) };
        return { params };
    }
    return null;
}

function bindAddRule() {
    document.getElementById('ar-add').addEventListener('click', () => {
        const type = document.getElementById('ar-new-type').value;
        const name = document.getElementById('ar-new-name').value.trim() || `${type} rule`;
        const rule = engine.newRule(type, name);
        state = engine.addRule(state, rule);
        engine.saveState(state);
        document.getElementById('ar-new-name').value = '';
        renderRulesList();
    });
}

function bindFeedButtons() {
    document.getElementById('ar-demo').addEventListener('click', () => {
        const { ticks, adv } = engine.makeDemoData();
        document.getElementById('ar-ticks').value =
            ticks.map(t => `${t.symbol} ${t.ts} ${t.price} ${t.volume}`).join('\n');
        document.getElementById('ar-adv').value =
            Object.entries(adv).map(([s, v]) => `${s} ${v}`).join('\n');
    });
    document.getElementById('ar-clear-feed').addEventListener('click', () => {
        document.getElementById('ar-ticks').value = '';
        document.getElementById('ar-adv').value = '';
    });
    document.getElementById('ar-replay').addEventListener('click', runReplay);
    document.getElementById('ar-stop-tts').addEventListener('click', () => {
        if (typeof window !== 'undefined' && window.speechSynthesis) {
            window.speechSynthesis.cancel();
        }
    });
}

function renderRulesList() {
    const wrap = document.getElementById('ar-rules');
    if (!state.rules.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.alert_rules.empty.rules">No rules. Add one below.</div>`;
        return;
    }
    wrap.innerHTML = state.rules.map(r => renderRuleCard(r)).join('');
}

function renderRuleCard(r) {
    const paramFields = paramsEditor(r);
    return `
        <div class="ar-rule">
            <div class="ar-rule-head">
                <label><input type="checkbox" data-rule-id="${esc(r.id)}" data-field="enabled" ${r.enabled ? 'checked' : ''}> enabled</label>
                <input type="text" data-rule-id="${esc(r.id)}" data-field="name" value="${esc(r.name)}" style="flex:1;min-width:160px">
                <span class="ar-rule-type">${esc(r.type)}</span>
                <button class="db-tile-btn db-tile-remove" data-rule-id="${esc(r.id)}" data-action="remove"
                        data-i18n-title="view.alert_rules.btn.remove_title" title="Remove">×</button>
            </div>
            <div class="ar-rule-body">
                <div class="inline-form">
                    <label><span data-i18n="view.alert_rules.label.sound">Sound</span>
                        <select data-rule-id="${esc(r.id)}" data-field="sound">
                            ${['none', 'bell', 'alarm', 'single_beep', 'double_beep'].map(s =>
                                `<option value="${s}" ${r.sound === s ? 'selected' : ''}>${s.replace('_', ' ')}</option>`).join('')}
                        </select></label>
                    <label><span data-i18n="view.alert_rules.label.cooldown_sec">Cooldown sec</span>
                        <input type="number" min="0" step="1" data-rule-id="${esc(r.id)}" data-field="cooldown_seconds" value="${r.cooldown_seconds}"></label>
                    <label><span data-i18n="view.alert_rules.label.watchlist">Watchlist (comma-sep, blank=all)</span>
                        <input type="text" data-rule-id="${esc(r.id)}" data-field="watchlist" value="${esc(r.watchlist.join(','))}" style="min-width:200px"></label>
                </div>
                <div class="inline-form">
                    ${paramFields}
                </div>
                <div class="inline-form">
                    <label style="flex:1"><span data-i18n="view.alert_rules.label.tts_template">TTS template (placeholders: {symbol} {price} {change_pct} {volume_mult} {threshold})</span>
                        <input type="text" data-rule-id="${esc(r.id)}" data-field="tts_template" value="${esc(r.tts_template || '')}" placeholder="optional; leave blank for default" data-i18n-placeholder="view.alert_rules.placeholder.tts" style="width:100%"></label>
                </div>
            </div>
        </div>
    `;
}

function paramsEditor(r) {
    const inp = (key, label, step, defaultV) => `
        <label>${esc(label)}
            <input type="number" step="${step}" data-rule-id="${esc(r.id)}" data-field="params.${key}" value="${esc(String(r.params[key] ?? defaultV))}">
        </label>
    `;
    switch (r.type) {
        case 'squeeze':
            return [
                inp('price_threshold_pct', 'price threshold (decimal)', 'any', 0.05),
                inp('volume_threshold',    'volume multiplier',         'any', 2.0),
                inp('window_seconds',      'window seconds',            '1',   300),
            ].join('');
        case 'price_above':
        case 'price_below':
            return inp('threshold', 'price threshold', 'any', 100);
        case 'pct_change':
            return [
                inp('pct_threshold',  'pct threshold (decimal)', 'any', 0.05),
                inp('window_seconds', 'window seconds',          '1',   300),
            ].join('');
        case 'volume_spike':
            return [
                inp('volume_mult',    'volume multiplier', 'any', 3.0),
                inp('window_seconds', 'window seconds',    '1',   300),
            ].join('');
        default:
            return '';
    }
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

async function runReplay() {
    const errs = document.getElementById('ar-errors');
    errs.style.display = 'none';
    const tickText = document.getElementById('ar-ticks').value;
    const advText = document.getElementById('ar-adv').value;
    const { ticks, errors: te } = squeeze.parseTickBlob(tickText);
    const { adv, errors: ae } = squeeze.parseAdvBlob(advText);
    const errors = [...te.map(e => ({ ...e, src: 'ticks' })), ...ae.map(e => ({ ...e, src: 'adv' }))];
    if (errors.length) {
        errs.innerHTML = `<strong>${errors.length} parse error(s):</strong><br>` +
            errors.slice(0, 8).map(e => `[${e.src}] line ${e.line_no}: ${esc(e.message)}`).join('<br>');
        errs.style.display = 'block';
        if (!ticks.length) return;
    }
    const events = engine.detectEvents(ticks, adv, state);
    renderEvents(events);
    // Fire audio in chronological order, paced at 0.2× real-time.
    const baseTs = events[0]?.ts;
    if (baseTs == null) return;
    for (const ev of events) {
        const offsetMs = Math.min((ev.ts - baseTs) * 200, 8000);
        setTimeout(() => fireAudioFor(ev), offsetMs);
    }
}

function fireAudioFor(ev) {
    audio.playSound(ev.sound, { volume: 0.3 });
    if (ev.message) {
        setTimeout(() => audio.speakAlert(ev.message, { volume: 0.6 }), 600);
    }
}

function renderEvents(events) {
    const wrap = document.getElementById('ar-events');
    if (!events.length) {
        wrap.innerHTML = `<div class="chart-panel"><div class="muted" data-i18n="view.alert_rules.empty.events">No events fired at current rule set.</div></div>`;
        return;
    }
    wrap.innerHTML = `
        <div class="chart-panel">
            <h2>${esc(tr('view.alert_rules.h2.fired_events', { count: events.length }))}</h2>
            <table class="lq-table">
                <thead><tr>
                    <th>#</th><th data-i18n="view.alert_rules.th.time">Time</th><th data-i18n="view.alert_rules.th.rule">Rule</th><th data-i18n="view.alert_rules.th.type">Type</th>
                    <th data-i18n="view.alert_rules.th.symbol">Symbol</th><th data-i18n="view.alert_rules.th.sound">Sound</th><th data-i18n="view.alert_rules.th.spoken_message">Spoken message</th>
                </tr></thead>
                <tbody>
                    ${events.map((e, i) => `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(new Date(e.ts * 1000).toISOString().slice(11, 19))}</td>
                        <td>${esc(e.rule_name)}</td>
                        <td>${esc(e.type)}</td>
                        <td><strong>${esc(e.symbol)}</strong></td>
                        <td>${esc(e.sound)}</td>
                        <td class="muted">"${esc(e.message)}"</td>
                    </tr>`).join('')}
                </tbody>
            </table>
        </div>
    `;
}
