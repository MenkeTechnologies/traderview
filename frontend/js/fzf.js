// fzf-style fuzzy filter + char-highlight engine.
//
// Port of audio_haxor/frontend/js/utils.js (lines ~214-562). Provides:
//   - `searchScore(query, fields, mode)` — score a row against fzf-extended
//     query syntax across multiple fields. 0 = no match.
//   - `searchMatch(query, fields, mode)` — boolean wrapper.
//   - `getMatchIndices(query, text, mode)` — set of char indices to highlight.
//   - `highlightWithIndices(text, indices)` — wrap matched chars in
//     `<mark class="fzf-hl">`.
//
// Query syntax (fzf extended):
//   foo bar           — both terms must match (fuzzy, case-insensitive)
//   ^foo              — prefix match on a field
//   foo$              — suffix match
//   'foo              — exact substring (no fuzzy)
//   !foo              — negate (row must NOT contain foo)
//   foo | bar         — OR group (either matches)
//
// Modes:
//   'fuzzy' (default) — full fzf syntax + scoring
//   'regex'           — query is a JS RegExp, case-insensitive
//
// CSS: callers should style `mark.fzf-hl` (see fzf.css for the default).

import { esc } from './util.js';

// ── scoring constants (fzf defaults; can be overridden via setFzfWeights) ──

const DEFAULTS = {
    SCORE_MATCH: 16,
    SCORE_GAP_START: -3,
    SCORE_GAP_EXTENSION: -1,
    BONUS_BOUNDARY: 9,
    BONUS_NON_WORD: 8,
    BONUS_CAMEL: 7,
    BONUS_CONSECUTIVE: 4,
    BONUS_FIRST_CHAR_MULT: 2,
};

const W = { ...DEFAULTS };

export function setFzfWeights(partial) {
    Object.assign(W, partial || {});
}
export function resetFzfWeights() {
    Object.assign(W, DEFAULTS);
}
export function getFzfWeights() {
    return { ...W };
}

const SCORE_SUBSTRING_BONUS = 1000;
const SCORE_EXACT_BONUS = 2000;
const SCORE_PREFIX_BONUS = 1500;

// ── char classes for boundary/camelCase bonuses ──

function charClass(c) {
    if (c >= 'a' && c <= 'z') return 1;
    if (c >= 'A' && c <= 'Z') return 2;
    if (c >= '0' && c <= '9') return 3;
    return 0;
}
function positionBonus(prev, curr) {
    const pc = charClass(prev);
    const cc = charClass(curr);
    if (pc === 0 && cc !== 0) return W.BONUS_BOUNDARY;
    if (pc === 1 && cc === 2) return W.BONUS_CAMEL;
    if (cc !== 0 && pc !== 0 && pc !== cc) return W.BONUS_NON_WORD;
    return 0;
}

// ── core fzf match — returns {score, indices} or null ──

export function fzfMatch(needle, haystack) {
    const nLen = needle.length, hLen = haystack.length;
    if (nLen === 0) return { score: 0, indices: [] };
    if (nLen > hLen) return null;

    const nLower = needle.toLowerCase();
    const hLower = haystack.toLowerCase();

    // Quick subsequence presence check.
    let ni = 0;
    for (let hi = 0; hi < hLen && ni < nLen; hi++) {
        if (hLower[hi] === nLower[ni]) ni++;
    }
    if (ni < nLen) return null;

    // For each starting position of needle[0], greedy-match the rest and
    // score; keep the highest-scoring placement. Quadratic worst-case but
    // bounded by needle length, which is tiny for filter inputs.
    let bestScore = -Infinity, bestIndices = null;
    const starts = [];
    for (let i = 0; i <= hLen - nLen; i++) {
        if (hLower[i] === nLower[0]) starts.push(i);
    }

    for (const start of starts) {
        const indices = [start];
        let si = start;
        let valid = true;

        for (let n = 1; n < nLen; n++) {
            let found = false;
            for (let h = si + 1; h < hLen; h++) {
                if (hLower[h] === nLower[n]) {
                    indices.push(h);
                    si = h;
                    found = true;
                    break;
                }
            }
            if (!found) { valid = false; break; }
        }
        if (!valid) continue;

        let score = 0;
        let prevIdx = -2;
        for (let i = 0; i < indices.length; i++) {
            const idx = indices[i];
            score += W.SCORE_MATCH;
            const prev = idx > 0 ? haystack[idx - 1] : ' ';
            let bonus = positionBonus(prev, haystack[idx]);
            if (i === 0) bonus *= W.BONUS_FIRST_CHAR_MULT;
            score += bonus;
            if (prevIdx === idx - 1) {
                score += W.BONUS_CONSECUTIVE;
            } else if (i > 0) {
                const gap = idx - prevIdx - 1;
                score += W.SCORE_GAP_START + W.SCORE_GAP_EXTENSION * (gap - 1);
            }
            prevIdx = idx;
        }

        if (score > bestScore) {
            bestScore = score;
            bestIndices = indices;
        }
    }

    if (!bestIndices) return null;
    return { score: bestScore, indices: bestIndices };
}

// ── extended-search parser ──

export function parseFzfQuery(query) {
    const tokens = String(query || '').split(/\s+/).filter(Boolean);
    const groups = [];
    let currentGroup = [];

    for (const token of tokens) {
        if (token === '|') continue;
        if (token.startsWith('|')) {
            currentGroup.push(parseToken(token.slice(1)));
        } else if (token.endsWith('|')) {
            currentGroup.push(parseToken(token.slice(0, -1)));
            groups.push(currentGroup);
            currentGroup = [];
        } else {
            if (currentGroup.length > 0) {
                groups.push(currentGroup);
                currentGroup = [];
            }
            currentGroup = [parseToken(token)];
        }
    }
    if (currentGroup.length > 0) groups.push(currentGroup);
    return groups;
}

function parseToken(token) {
    let negate = false, type = 'fuzzy', text = token;
    if (text.startsWith('!')) {
        negate = true;
        text = text.slice(1);
    }
    if (text.startsWith("'") && text.endsWith("'") && text.length > 2) {
        type = 'exact';
        text = text.slice(1, -1);
    } else if (text.startsWith("'")) {
        type = 'exact';
        text = text.slice(1);
    } else if (text.startsWith('^')) {
        type = 'prefix';
        text = text.slice(1);
    } else if (text.endsWith('$')) {
        type = 'suffix';
        text = text.slice(0, -1);
    }
    return { type, text, negate };
}

function scoreToken(token, value) {
    const v = value.toLowerCase(), t = token.text.toLowerCase();
    switch (token.type) {
        case 'exact':
            return v.includes(t) ? SCORE_SUBSTRING_BONUS + t.length * W.SCORE_MATCH : 0;
        case 'prefix':
            return v.startsWith(t) ? SCORE_PREFIX_BONUS + t.length * W.SCORE_MATCH : 0;
        case 'suffix':
            return v.endsWith(t) ? SCORE_SUBSTRING_BONUS + t.length * W.SCORE_MATCH : 0;
        case 'fuzzy': {
            if (v === t) return SCORE_EXACT_BONUS + t.length * W.SCORE_MATCH;
            if (v.includes(t)) return SCORE_SUBSTRING_BONUS + t.length * W.SCORE_MATCH;
            const m = fzfMatch(token.text, value);
            return m ? m.score : 0;
        }
    }
    return 0;
}

// ── public: match rows against an extended-search query ──

export function searchScore(query, fields, mode) {
    if (!query) return 1;
    const list = Array.isArray(fields) ? fields : [fields];
    if (mode === 'regex') {
        try {
            const re = new RegExp(query, 'i');
            return list.some(f => re.test(String(f || ''))) ? 1 : 0;
        } catch {
            const q = String(query).toLowerCase();
            return list.some(f => String(f || '').toLowerCase().includes(q)) ? 1 : 0;
        }
    }
    const groups = parseFzfQuery(query);
    let totalScore = 0;
    for (const orGroup of groups) {
        let bestGroupScore = 0;
        for (const token of orGroup) {
            let tokenBest = 0;
            for (let fi = 0; fi < list.length; fi++) {
                // First field treated as the primary label — extra weight.
                const fieldBonus = fi === 0 ? 500 : 0;
                const s = scoreToken(token, String(list[fi] || ''));
                if (s > 0 && s + fieldBonus > tokenBest) tokenBest = s + fieldBonus;
            }
            if (token.negate) {
                if (tokenBest > 0) return 0;
                bestGroupScore = 1;
            } else {
                if (tokenBest > bestGroupScore) bestGroupScore = tokenBest;
            }
        }
        if (bestGroupScore === 0) return 0;
        totalScore += bestGroupScore;
    }
    return totalScore;
}

export function searchMatch(query, fields, mode) {
    return searchScore(query, fields, mode) > 0;
}

// ── public: indices to highlight in one field ──

export function getMatchIndices(query, text, mode) {
    if (!query || !text) return [];
    if (mode === 'regex') {
        try {
            const re = new RegExp(query, 'ig');
            const indices = [];
            let m;
            while ((m = re.exec(text)) !== null) {
                for (let i = m.index; i < m.index + m[0].length; i++) indices.push(i);
                if (m.index === re.lastIndex) re.lastIndex++; // zero-length match guard
            }
            return indices;
        } catch {
            return [];
        }
    }
    const groups = parseFzfQuery(query);
    const allIndices = new Set();
    for (const group of groups) {
        for (const token of group) {
            if (token.negate) continue;
            if (token.type === 'fuzzy') {
                const m = fzfMatch(token.text, text);
                if (m) m.indices.forEach(i => allIndices.add(i));
            } else {
                const tLower = token.text.toLowerCase();
                const lower = text.toLowerCase();
                let from = 0;
                if (token.type === 'prefix') {
                    if (lower.startsWith(tLower)) {
                        for (let i = 0; i < tLower.length; i++) allIndices.add(i);
                    }
                } else if (token.type === 'suffix') {
                    if (lower.endsWith(tLower)) {
                        const start = text.length - tLower.length;
                        for (let i = 0; i < tLower.length; i++) allIndices.add(start + i);
                    }
                } else {
                    // exact / fallback: every occurrence
                    while (true) {
                        const idx = lower.indexOf(tLower, from);
                        if (idx < 0) break;
                        for (let i = idx; i < idx + tLower.length; i++) allIndices.add(i);
                        from = idx + Math.max(1, tLower.length);
                    }
                }
            }
        }
    }
    return [...allIndices].sort((a, b) => a - b);
}

// ── public: wrap matched chars in <mark class="fzf-hl"> ──

export function highlightWithIndices(text, indices) {
    if (text == null) return '';
    const str = String(text);
    if (!indices || indices.length === 0) return esc(str);
    const idxSet = new Set(indices);
    let result = '';
    let inMark = false;
    for (let i = 0; i < str.length; i++) {
        const ch = esc(str[i]);
        if (idxSet.has(i)) {
            if (!inMark) { result += '<mark class="fzf-hl">'; inMark = true; }
            result += ch;
        } else {
            if (inMark) { result += '</mark>'; inMark = false; }
            result += ch;
        }
    }
    if (inMark) result += '</mark>';
    return result;
}

// Convenience: score + highlight in one shot.
// Returns { score, html } where html is the highlighted first field.
export function searchScoreHighlight(query, fields, mode) {
    const list = Array.isArray(fields) ? fields : [fields];
    const score = searchScore(query, list, mode);
    if (score === 0) return { score: 0, html: esc(String(list[0] || '')) };
    const indices = getMatchIndices(query, String(list[0] || ''), mode);
    return { score, html: highlightWithIndices(String(list[0] || ''), indices) };
}
