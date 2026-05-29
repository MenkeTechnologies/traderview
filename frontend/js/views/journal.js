import { api } from '../api.js';
import { fmtDateTime, md, esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderJournalView(mount, _state, dayOrGeneral) {
    const tok = currentViewToken();
    const isGeneral = dayOrGeneral === 'general';
    const day = isGeneral ? null : (dayOrGeneral || new Date().toISOString().slice(0, 10));
    const entries = isGeneral
        ? await api.journalGeneral()
        : await api.journalForDay(day);
    if (!viewIsCurrent(tok)) return;

    mount.innerHTML = `
        <h1 class="view-title">
            // JOURNAL ·
            ${isGeneral
                ? `<span style="color:var(--magenta)">${esc(t('view.journal.label.general'))}</span>`
                : `<input type="date" id="journal-day" value="${day}">`}
            <a href="#journal/${isGeneral ? new Date().toISOString().slice(0,10) : 'general'}" class="link small">
                ${esc(t(isGeneral ? 'view.journal.link.switch_to_daily' : 'view.journal.link.switch_to_general'))}
            </a>
        </h1>
        <div id="entries">${entries.map(e => `
            <div class="journal-entry">
                <div class="meta">
                    ${fmtDateTime(e.created_at)}
                    ${e.mood !== null ? `· mood ${e.mood}` : ''}
                    ${e.trade_id ? `· <a href="#trade/${e.trade_id}">trade</a>` : ''}
                </div>
                <div class="body">${md(e.body_md)}</div>
                <button data-i18n="view.journal.btn.delete" class="link" data-del="${e.id}">delete</button>
            </div>
        `).join('') || `<p class="muted">${esc(t(isGeneral ? 'view.journal.empty.general' : 'view.journal.empty.day'))}</p>`}</div>
        <div class="chart-panel">
            <h2 data-i18n="view.journal.h2.new_entry">New entry</h2>
            ${isGeneral ? '' : `
                <select id="mood">
                    <option data-i18n="view.journal.opt.no_mood" value="">no mood</option>
                    <option data-i18n="view.journal.opt.2_frustrated" value="-2">-2 frustrated</option>
                    <option data-i18n="view.journal.opt.1_off" value="-1">-1 off</option>
                    <option data-i18n="view.journal.opt.0_neutral" value="0">0 neutral</option>
                    <option data-i18n="view.journal.opt.1_focused" value="1">+1 focused</option>
                    <option data-i18n="view.journal.opt.2_confident" value="2">+2 confident</option>
                </select>
            `}
            <textarea id="body" placeholder="What happened today? Setups taken / missed? Process notes?"
                      data-i18n-placeholder="view.journal.placeholder.body"></textarea>
            <div class="inline-form">
                <button data-i18n="view.journal.btn.save" class="primary" id="save">Save</button>
                <button data-i18n="view.journal.btn.insert_template" class="primary" id="apply-template" style="background:linear-gradient(180deg,var(--magenta),#7f00b5);border-color:var(--magenta)">Insert template</button>
            </div>
        </div>
    `;
    const dayInput = mount.querySelector('#journal-day');
    if (dayInput) {
        dayInput.addEventListener('change', (e) => {
            window.location.hash = `journal/${e.target.value}`;
        });
    }
    mount.querySelector('#save').addEventListener('click', async () => {
        const body_md = mount.querySelector('#body').value.trim();
        if (!body_md) return;
        const mood = mount.querySelector('#mood')?.value;
        await api.createJournal({
            day: isGeneral ? null : day,
            body_md,
            mood: mood === '' || mood === undefined ? null : Number(mood),
        });
        if (!viewIsCurrent(tok)) return;
        renderJournalView(mount, _state, dayOrGeneral);
    });
    mount.querySelector('#apply-template').addEventListener('click', async () => {
        const tpl = await api.defaultNoteTemplate('journal');
        if (!viewIsCurrent(tok)) return;
        const ta = mount.querySelector('#body');
        if (!ta) return;
        if (tpl && tpl.body_md) {
            ta.value = (ta.value ? ta.value + '\n\n' : '') + tpl.body_md;
        } else {
            alert(t('view.journal.alert.no_template'));
        }
    });
    mount.querySelectorAll('[data-del]').forEach(b =>
        b.addEventListener('click', async () => {
            await api.deleteJournal(b.dataset.del);
            if (!viewIsCurrent(tok)) return;
            renderJournalView(mount, _state, dayOrGeneral);
        }));
    void esc;
}
