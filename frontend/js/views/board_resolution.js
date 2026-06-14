// Board resolution generator — quorum + vote tally → passed/failed, via
// /calc/board-resolution. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderBoardResolution(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.br.h1.title">// BOARD RESOLUTION</span></h1>
        <p class="muted small" data-i18n="view.br.hint.intro">
            Records a decision of a company's board of directors. It checks whether a quorum was present
            (more than half the directors) and tallies the vote (for / against) to determine whether the
            resolution passed. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.br.h2.inputs">Resolution details</h2>
            <form id="br-form" class="inline-form">
                <label><span data-i18n="view.br.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.br.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.br.label.date">Meeting date</span>
                    <input type="date" name="meeting_date" value="2026-07-20" required></label>
                <label><span data-i18n="view.br.label.subject">Subject</span>
                    <input type="text" name="resolution_subject" value="Approval of stock issuance"></label>
                <label><span data-i18n="view.br.label.text">Resolution text (RESOLVED, that …)</span>
                    <input type="text" name="resolution_text" value="the Company is authorized to issue 100,000 shares of common stock to the investor on the agreed terms."></label>
                <label><span data-i18n="view.br.label.total">Total directors</span>
                    <input type="number" step="1" min="1" name="total_directors" value="5" required></label>
                <label><span data-i18n="view.br.label.present">Directors present</span>
                    <input type="number" step="1" min="0" name="directors_present" value="4" required></label>
                <label><span data-i18n="view.br.label.for">Votes for</span>
                    <input type="number" step="1" min="0" name="votes_for" value="3" required></label>
                <label><span data-i18n="view.br.label.against">Votes against</span>
                    <input type="number" step="1" min="0" name="votes_against" value="1"></label>
                <label><span data-i18n="view.br.label.abstain">Votes abstain</span>
                    <input type="number" step="1" min="0" name="votes_abstain" value="0"></label>
            </form>
        </div>
        <div id="br-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#br-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            meeting_date: fd.get('meeting_date'),
            resolution_subject: (fd.get('resolution_subject') || '').trim(),
            resolution_text: (fd.get('resolution_text') || '').trim(),
            total_directors: Math.round(Number(fd.get('total_directors')) || 0),
            directors_present: Math.round(Number(fd.get('directors_present')) || 0),
            votes_for: Math.round(Number(fd.get('votes_for')) || 0),
            votes_against: Math.round(Number(fd.get('votes_against')) || 0),
            votes_abstain: Math.round(Number(fd.get('votes_abstain')) || 0),
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcBoardResolution(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.br.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#br-result');
    const resultKey = doc.passed ? 'view.br.status.passed' : 'view.br.status.failed';
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${doc.passed ? 'pos' : 'neg'}"><div class="label" data-i18n="view.br.card.result">Result</div>
                    <div class="value" data-i18n="${resultKey}"></div></div>
                <div class="card ${doc.quorum_met ? 'pos' : 'neg'}"><div class="label" data-i18n="view.br.card.quorum">Quorum</div>
                    <div class="value" data-i18n="${doc.quorum_met ? 'view.br.quorum.yes' : 'view.br.quorum.no'}"></div></div>
                <div class="card"><div class="label" data-i18n="view.br.card.vote">Vote</div>
                    <div class="value">${doc.votes_for}–${doc.votes_against}–${doc.votes_abstain}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="br-copy" type="button" data-i18n="view.br.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="br-download" type="button" data-i18n="view.br.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#br-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.br.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.br.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#br-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'board-resolution.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
