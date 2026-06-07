import { api } from '../api.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderPlans(mount, state) {
    const tok = currentViewToken();
    if (!state.accountId) { mount.innerHTML = '<p data-i18n="view.plans.hint.no_account" class="boot">No account.</p>'; return; }
    const plans = await api.plans();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.plans.h1.pre_trade_plans" class="view-title">// PRE-TRADE PLANS</h1>
        <div class="chart-panel">
            <h2 data-i18n="view.plans.h2.new_plan">New plan</h2>
            <form id="plan-form" class="inline-form">
                <input name="symbol" data-shortcut="focus_search" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" required>
                <select name="asset_class">
                    <option data-i18n="view.plans.opt.stock" value="stock">stock</option>
                    <option data-i18n="view.plans.opt.option" value="option">option</option>
                    <option data-i18n="view.plans.opt.future" value="future">future</option>
                    <option data-i18n="view.plans.opt.forex" value="forex">forex</option>
                </select>
                <select name="side"><option data-i18n="view.plans.opt.long" value="long">long</option><option data-i18n="view.plans.opt.short" value="short">short</option></select>
                <input name="intended_qty" type="number" step="0.01" placeholder="qty" data-i18n-placeholder="common.placeholder.qty" required>
                <input name="intended_entry" type="number" step="0.01" placeholder="entry" data-i18n-placeholder="common.placeholder.entry" required>
                <input name="stop_loss" type="number" step="0.01" placeholder="stop" data-i18n-placeholder="common.placeholder.stop">
                <input name="initial_target" type="number" step="0.01" placeholder="target" data-i18n-placeholder="common.placeholder.target">
                <input name="setup_notes" placeholder="setup notes" data-i18n-placeholder="view.plans.placeholder.setup_notes">
                <button data-i18n="view.plans.btn.create" class="primary" type="submit">Create</button>
            </form>
        </div>

        <table class="trades">
            <thead><tr>
                <th data-i18n="view.plans.th.created">Created</th><th data-i18n="view.plans.th.symbol">Symbol</th><th data-i18n="view.plans.th.side">Side</th><th data-i18n="view.plans.th.qty">Qty</th>
                <th data-i18n="view.plans.th.entry">Entry</th><th data-i18n="view.plans.th.stop">Stop</th><th data-i18n="view.plans.th.target">Target</th><th data-i18n="view.plans.th.r_r">R:R</th><th data-i18n="view.plans.th.setup">Setup</th><th></th>
            </tr></thead>
            <tbody>${plans.map(p => {
                const risk = p.stop_loss ? Math.abs(Number(p.intended_entry) - Number(p.stop_loss)) : null;
                const reward = p.initial_target ? Math.abs(Number(p.initial_target) - Number(p.intended_entry)) : null;
                const rr = risk && reward ? (reward / risk).toFixed(2) : '—';
                return `<tr data-context-scope="plan-row" data-id="${esc(p.id)}" data-symbol="${esc(p.symbol)}">
                    <td>${fmtDateTime(p.created_at)}</td>
                    <td>${esc(p.symbol)}</td>
                    <td>${p.side}</td>
                    <td>${fmt(p.intended_qty, 0)}</td>
                    <td>${fmt(p.intended_entry)}</td>
                    <td>${p.stop_loss !== null ? fmt(p.stop_loss) : '—'}</td>
                    <td>${p.initial_target !== null ? fmt(p.initial_target) : '—'}</td>
                    <td>${rr}</td>
                    <td>${esc(p.setup_notes)}</td>
                    <td><button data-i18n="view.plans.btn.abandon" class="link" data-del="${p.id}">abandon</button></td>
                </tr>`;
            }).join('') || `<tr><td colspan="10" class="muted">${esc(t('view.plans.empty'))}</td></tr>`}
            </tbody>
        </table>

        <div class="chart-panel">
            <h2 data-i18n="view.plans.h2.rr_chart">R:R per plan (sorted desc)</h2>
            <div id="plans-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.plans.h2.risk_chart">Stop distance per plan (sorted widest first)</h2>
            <div id="plans-risk-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.plans.hint.risk_chart" class="muted small">|entry − stop| per share. Wide stops mean smaller position for a fixed risk budget; tight stops mean larger. Critical for sizing decisions across the plan set.</p>
        </div>
    `;
    renderRrChart(plans);
    renderRiskChart(plans);
    mount.querySelector('#plan-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            account_id: state.accountId,
            symbol: fd.get('symbol'),
            asset_class: fd.get('asset_class'),
            side: fd.get('side'),
            intended_qty: Number(fd.get('intended_qty')),
            intended_entry: Number(fd.get('intended_entry')),
            stop_loss: fd.get('stop_loss') ? Number(fd.get('stop_loss')) : null,
            initial_target: fd.get('initial_target') ? Number(fd.get('initial_target')) : null,
            setup_notes: fd.get('setup_notes') || '',
        };
        await api.createPlan(body);
        if (!viewIsCurrent(tok)) return;
        renderPlans(mount, state);
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.abandonPlan(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderPlans(mount, state);
        }));
}

function renderRrChart(plans) {
    const el = document.getElementById('plans-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (plans || []).map(p => {
        const risk = p.stop_loss ? Math.abs(Number(p.intended_entry) - Number(p.stop_loss)) : null;
        const reward = p.initial_target ? Math.abs(Number(p.initial_target) - Number(p.intended_entry)) : null;
        if (!Number.isFinite(risk) || !Number.isFinite(reward) || risk <= 0) return null;
        return { symbol: p.symbol, rr: reward / risk };
    }).filter(Boolean).sort((a, b) => b.rr - a.rr).slice(0, 30);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.plans.empty_chart">${esc(t('view.plans.empty_chart'))}</div>`;
        return;
    }
    const labels = rows.map(r => r.symbol);
    const ys = rows.map(r => r.rr);
    const xs = labels.map((_, i) => i + 1);
    const target = xs.map(() => 2);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.plans.chart.plan_idx') },
            { label: t('view.plans.chart.rr'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.plans.chart.target'),
              stroke: '#7af0a8', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys, target], el);
}

function renderRiskChart(plans) {
    const el = document.getElementById('plans-risk-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const rows = (plans || []).map(p => {
        if (!p.stop_loss) return null;
        const risk = Math.abs(Number(p.intended_entry) - Number(p.stop_loss));
        return Number.isFinite(risk) && risk > 0 ? { symbol: p.symbol, risk } : null;
    }).filter(Boolean).sort((a, b) => b.risk - a.risk).slice(0, 30);
    if (rows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.plans.empty_risk_chart">${esc(t('view.plans.empty_risk_chart'))}</div>`;
        return;
    }
    const labels = rows.map(r => r.symbol);
    const ys = rows.map(r => r.risk);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.plans.chart.plan_idx') },
            { label: t('view.plans.chart.stop_dist'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
