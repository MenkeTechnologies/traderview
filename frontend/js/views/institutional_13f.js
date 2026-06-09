// 13F Institutional Portfolio — Finnhub /institutional/portfolio + /profile.
// Browse hedge fund holdings by CIK. Famous CIKs: Berkshire 0001067983,
// Renaissance 0001037389, Bridgewater 0001350694, Soros 0001029160.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const FAMOUS = [
    { cik: '0001067983', label: 'Berkshire Hathaway (Buffett)' },
    { cik: '0001037389', label: 'Renaissance Technologies (Simons)' },
    { cik: '0001350694', label: 'Bridgewater (Dalio)' },
    { cik: '0001029160', label: 'Soros Fund Management' },
    { cik: '0001364742', label: 'Citadel Advisors (Griffin)' },
    { cik: '0001167483', label: 'Tudor Investment (Jones)' },
    { cik: '0001037389', label: 'Two Sigma' },
    { cik: '0001423053', label: 'Pershing Square (Ackman)' },
    { cik: '0001403438', label: 'Greenlight Capital (Einhorn)' },
    { cik: '0001063296', label: 'Third Point (Loeb)' },
];

let state = { cik: '0001067983' };

export async function renderInstitutional13F(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.inst13f.h1.title">// 13F INSTITUTIONAL PORTFOLIO</span></h1>
        <p class="muted small" data-i18n="view.inst13f.hint.intro">
            Hedge fund 13F holdings via Finnhub. Filed quarterly within 45 days of quarter
            end — data is lagged 45-90 days. Smart-money positioning, not actionable signals.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="i13f-form">
                <label><span data-i18n="view.inst13f.label.fund">Famous fund</span>
                    <select id="i13f-famous">
                        <option value="" data-i18n="view.inst13f.opt.custom">(custom CIK)</option>
                        ${FAMOUS.map(f =>
                            `<option value="${esc(f.cik)}" ${f.cik === state.cik ? 'selected' : ''}>${esc(f.label)}</option>`
                        ).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.inst13f.label.cik">CIK</span>
                    <input type="text" name="cik" value="${esc(state.cik)}" placeholder="0001067983"></label>
                <button class="primary" type="submit" data-i18n="view.inst13f.btn.load">Load</button>
            </form>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.inst13f.h2.profile">Fund profile</h2>
                <div id="i13f-profile"></div>
            </div>
            <div class="chart-panel" style="grid-column:1/-1">
                <h2 data-i18n="view.inst13f.h2.portfolio">Latest portfolio</h2>
                <div id="i13f-portfolio"></div>
            </div>
        </div>
    `;
    document.getElementById('i13f-famous').addEventListener('change', e => {
        if (e.target.value) {
            state.cik = e.target.value;
            mount.querySelector('input[name="cik"]').value = state.cik;
            void load(tok);
        }
    });
    document.getElementById('i13f-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.cik = (fd.get('cik') || '').trim();
        void load(tok);
    });
    if (state.cik) await load(tok);
}

async function load(tok) {
    const profEl = document.getElementById('i13f-profile');
    const portEl = document.getElementById('i13f-portfolio');
    [profEl, portEl].forEach(el => el && (el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`));
    try {
        const to = new Date();
        const from = new Date(to);
        from.setFullYear(from.getFullYear() - 1);
        const [prof, port] = await Promise.all([
            api.finnhubInstProfile(state.cik).catch(() => null),
            api.finnhubInstPortfolio(state.cik, fmtDay(from), fmtDay(to)).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderProfile(profEl, prof);
        renderPortfolio(portEl, port);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.inst13f.toast.failed'), { level: 'error' });
    }
}

function renderProfile(el, p) {
    if (!el) return;
    const data = p?.data || p || {};
    if (!Object.keys(data).length) {
        el.innerHTML = `<p class="muted" data-i18n="view.inst13f.empty.profile">No profile.</p>`;
        return;
    }
    const rows = [
        [t('view.inst13f.profile.name'),    data.name],
        [t('view.inst13f.profile.cik'),     data.cik],
        [t('view.inst13f.profile.address'), data.address],
        [t('view.inst13f.profile.city'),    data.city],
        [t('view.inst13f.profile.state'),   data.state],
        [t('view.inst13f.profile.zip'),     data.zip],
    ];
    el.innerHTML = `<table class="trades"><tbody>${rows
        .filter(([_, v]) => v != null && v !== '')
        .map(([k, v]) => `<tr><td>${k}</td><td>${esc(String(v))}</td></tr>`)
        .join('')}</tbody></table>`;
}

function renderPortfolio(el, p) {
    if (!el) return;
    const filings = p?.data || [];
    if (!filings.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.inst13f.empty.portfolio">No portfolio data.</p>`;
        return;
    }
    // Take the most recent filing.
    const latest = [...filings].sort((a, b) =>
        String(b.reportDate || '').localeCompare(String(a.reportDate || '')))[0];
    const positions = latest?.portfolio || [];
    if (!positions.length) {
        el.innerHTML = `<p class="muted">${esc(t('view.inst13f.empty.positions', { date: latest?.reportDate || '?' }))}</p>`;
        return;
    }
    const totalValue = positions.reduce((s, p) => s + (Number(p.marketValue) || 0), 0);
    const sorted = [...positions].sort((a, b) =>
        (Number(b.marketValue) || 0) - (Number(a.marketValue) || 0));
    el.innerHTML = `
        <p class="muted small">
            <span data-i18n="view.inst13f.label.report_date">Report:</span> <strong>${esc(latest.reportDate || '—')}</strong>
            · <span data-i18n="view.inst13f.label.positions">Positions:</span> ${positions.length}
            · <span data-i18n="view.inst13f.label.aum">Total AUM:</span> $${(totalValue / 1e9).toFixed(2)}B
        </p>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.inst13f.th.rank">#</th>
                <th data-i18n="view.inst13f.th.symbol">Symbol</th>
                <th data-i18n="view.inst13f.th.name">Name</th>
                <th data-i18n="view.inst13f.th.shares">Shares</th>
                <th data-i18n="view.inst13f.th.value">Market value</th>
                <th data-i18n="view.inst13f.th.weight">Weight</th>
                <th data-i18n="view.inst13f.th.change">Δ shares</th>
            </tr></thead>
            <tbody>${sorted.slice(0, 100).map((p, i) => {
                const v = Number(p.marketValue) || 0;
                const w = totalValue > 0 ? (v / totalValue * 100) : 0;
                const ch = Number(p.changedShares || p.change || 0);
                const cls = ch > 0 ? 'pos' : ch < 0 ? 'neg' : '';
                return `<tr>
                    <td class="muted">${i + 1}</td>
                    <td><a class="link" href="#research/${esc(p.symbol || '')}">${esc(p.symbol || '—')}</a></td>
                    <td class="muted">${esc(p.name || '—')}</td>
                    <td>${Number(p.share || p.shares || 0).toLocaleString()}</td>
                    <td>$${(v / 1e6).toFixed(1)}M</td>
                    <td>${w.toFixed(2)}%</td>
                    <td class="${cls}">${ch ? (ch > 0 ? '+' : '') + ch.toLocaleString() : '—'}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>
    `;
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
