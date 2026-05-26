// Common UI helpers reused across views.

export const fmt = (n, d = 2) => {
    if (n === null || n === undefined || n === '') return '—';
    const v = Number(n);
    if (!Number.isFinite(v)) return '∞';
    return v.toLocaleString(undefined, { minimumFractionDigits: d, maximumFractionDigits: d });
};

export const fmtPct = (n) => {
    if (n === null || n === undefined) return '—';
    return (Number(n) * 100).toFixed(1) + '%';
};

export const fmtMoney = (n) => '$' + fmt(n);

export const pnlClass = (n) => Number(n) >= 0 ? 'pos' : 'neg';

export const fmtDate = (iso) => (iso || '').slice(0, 10);
export const fmtDateTime = (iso) =>
    iso ? new Date(iso).toLocaleString(undefined, { hour12: false }) : '—';

export const fmtSecs = (s) => {
    if (s === null || s === undefined) return '—';
    s = Number(s);
    if (s < 60) return `${s}s`;
    if (s < 3600) return `${(s / 60).toFixed(1)}m`;
    if (s < 86400) return `${(s / 3600).toFixed(1)}h`;
    return `${(s / 86400).toFixed(1)}d`;
};

export const html = (strings, ...values) => {
    let out = '';
    strings.forEach((s, i) => { out += s + (values[i] !== undefined ? values[i] : ''); });
    return out;
};

export const esc = (s) => String(s == null ? '' : s)
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;').replace(/'/g, '&#39;');

/// Tiny markdown renderer — bold/italic/code/links + paragraphs/headings/lists.
/// Not a full CommonMark; we only need basic safe rendering for journal/forum.
export const md = (src) => {
    const lines = String(src || '').split(/\r?\n/);
    const blocks = [];
    let para = [];
    const flushPara = () => {
        if (para.length) {
            blocks.push('<p>' + para.join(' ').trim() + '</p>');
            para = [];
        }
    };
    for (let line of lines) {
        line = line || '';
        if (/^\s*$/.test(line)) { flushPara(); continue; }
        if (/^###\s+/.test(line)) { flushPara(); blocks.push(`<h3>${esc(line.replace(/^###\s+/, ''))}</h3>`); continue; }
        if (/^##\s+/.test(line)) { flushPara(); blocks.push(`<h2>${esc(line.replace(/^##\s+/, ''))}</h2>`); continue; }
        if (/^#\s+/.test(line)) { flushPara(); blocks.push(`<h1>${esc(line.replace(/^#\s+/, ''))}</h1>`); continue; }
        if (/^[-*]\s+/.test(line)) {
            flushPara();
            const items = [line];
            while (lines[0] !== undefined && /^[-*]\s+/.test(lines[0])) items.push(lines.shift());
            blocks.push('<ul>' + items.map(i =>
                '<li>' + inline(i.replace(/^[-*]\s+/, '')) + '</li>'
            ).join('') + '</ul>');
            continue;
        }
        para.push(inline(line));
    }
    flushPara();
    return blocks.join('\n');
};

function inline(s) {
    return esc(s)
        .replace(/`([^`]+)`/g, '<code>$1</code>')
        .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
        .replace(/\*([^*]+)\*/g, '<em>$1</em>')
        .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>');
}

export function makeFilter(initial = {}, onApply) {
    const v = Object.assign({
        symbol: '', side: '', status: '', asset_class: '', date_from: '', date_to: '',
        min_pnl: '', max_pnl: '',
    }, initial);
    const wrap = document.createElement('div');
    wrap.className = 'filter-bar';
    wrap.innerHTML = `
        <input type="text"   name="symbol"      placeholder="symbol" value="${v.symbol}">
        <select name="side"><option value="">side</option>
            <option value="long" ${v.side === 'long' ? 'selected' : ''}>long</option>
            <option value="short" ${v.side === 'short' ? 'selected' : ''}>short</option>
        </select>
        <select name="status"><option value="">status</option>
            <option value="open" ${v.status === 'open' ? 'selected' : ''}>open</option>
            <option value="closed" ${v.status === 'closed' ? 'selected' : ''}>closed</option>
        </select>
        <select name="asset_class"><option value="">asset</option>
            <option value="stock" ${v.asset_class === 'stock' ? 'selected' : ''}>stock</option>
            <option value="option" ${v.asset_class === 'option' ? 'selected' : ''}>option</option>
            <option value="future" ${v.asset_class === 'future' ? 'selected' : ''}>future</option>
            <option value="forex" ${v.asset_class === 'forex' ? 'selected' : ''}>forex</option>
        </select>
        <input type="date" name="date_from" value="${v.date_from}">
        <input type="date" name="date_to" value="${v.date_to}">
        <input type="number" step="any" name="min_pnl" placeholder="min P&amp;L" value="${v.min_pnl}">
        <input type="number" step="any" name="max_pnl" placeholder="max P&amp;L" value="${v.max_pnl}">
        <button type="button" class="primary">Apply</button>
    `;
    const collect = () => {
        const f = {};
        wrap.querySelectorAll('input, select').forEach(el => {
            if (el.value !== '') f[el.name] = el.value;
        });
        return f;
    };
    wrap.querySelector('button').addEventListener('click', () => onApply(collect()));
    return { el: wrap, collect };
}

export function statCard(label, value, mod = '') {
    return `<div class="card"><div class="label">${esc(label)}</div>
        <div class="value ${mod}">${value}</div></div>`;
}
