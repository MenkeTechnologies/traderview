#!/usr/bin/env node
// i18n_extract_proper_nouns.mjs — walk app_i18n_en.json and harvest every
// proper-noun candidate: brand names, tickers, acronyms, regulator
// callouts, form numbers, CLI flags. Writes the deduped, sorted list to
// frontend/i18n/proper_nouns.txt as one term per line.
//
// The list is the source of truth for the DNT classifier: the translator
// masks occurrences with opaque sentinels before sending to LibreTranslate
// and restores them after — so "Compute BOP" → "Vypočítat BOP", not
// "Doba výpočtu". Run once after the EN file changes; check the diff
// into git so future translators benefit.
//
// Usage:
//   node frontend/scripts/i18n_extract_proper_nouns.mjs

import { readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const EN_PATH = resolve(ROOT, 'i18n/app_i18n_en.json');
const OUT_PATH = resolve(ROOT, 'i18n/proper_nouns.txt');

// Common ALL_CAPS / TitleCase English words that are NOT proper nouns and
// should be excluded. These appear inside i18n values legitimately but
// would be matched by our generic patterns. Add aggressively — false
// positives in this stoplist just mean the term goes back to being
// masked, which is the safer default.
const STOP_WORDS = new Set([
    'I', 'A', 'AN', 'THE', 'OR', 'AND', 'BUT', 'IF', 'THEN', 'ELSE',
    'IS', 'ARE', 'WAS', 'WERE', 'BE', 'BEEN', 'BEING',
    'NO', 'YES', 'OK', 'OFF', 'ON', 'UP', 'DOWN',
    'NEW', 'OLD', 'ALL', 'ANY', 'NONE', 'SOME',
    'TODO', 'TBD', 'NOTE', 'WARNING', 'ERROR', 'INFO', 'DEBUG',
    'TIP', 'HINT', 'PRO', 'TIPS',
    'MIN', 'MAX', 'AVG', 'SUM', 'TOTAL', 'NET',
    'TODAY', 'YESTERDAY', 'TOMORROW',
    'NOT', 'NEVER', 'ALWAYS', 'MAYBE', 'BUT', 'AND', 'OR',
    'YOU', 'WE', 'US', 'THEY', 'HIM', 'HER', 'HIS', 'OUR',
    'PM', 'AM', 'BC', 'AD', 'ET', 'PT', 'CT', 'GMT', 'UTC',
    'USD', 'CAD', 'EUR', 'GBP', 'JPY', 'CHF', 'AUD', 'NZD',
    'OS', 'IP', 'URL', 'JSON', 'CSV', 'PDF', 'PNG', 'JPG', 'JPEG', 'HEIC',
    'API', 'SDK', 'CLI', 'GUI', 'TUI', 'IDE', 'VM',
    // Trading/finance acronyms used AS terms (might still be DNT —
    // include here so they DON'T get aggressively whole-string DNT'd via
    // patterns; their treatment is handled by the dedicated section
    // below in the extractor's `KNOWN_TERMS` list).
]);

// Patterns for STRUCTURAL proper-noun detection. Each yields tokens that
// are filtered against STOP_WORDS. We deliberately do NOT match plain
// TitleCase words (Demo, Compute, Year, etc.) because they're frequently
// common English and cause massive false positives — unknown brand names
// instead go through KNOWN_TERMS, which is curated.
const PATTERNS = [
    // ALL_CAPS acronyms 3-7 chars (FINRA, NASDAQ, SIPC, VWAP).
    { name: 'acronym', re: /\b[A-Z]{3,7}\b/g },
    // Internal-cap brand names — at least one internal capital after the
    // first character, length 4-20 (PostgreSQL, OpenCV, MacOS, TraderView).
    // The internal capital is the disambiguating signal vs ordinary
    // sentence-starting TitleCase words.
    { name: 'camelcap', re: /\b[A-Z][a-z]+[A-Z][A-Za-z]{1,15}\b/g },
    // Mixed alpha/star (E*TRADE).
    { name: 'asterisk', re: /\b[A-Za-z]+\*[A-Z]+\b/g },
    // CLI flag tokens: --psm, --release. Require not preceded by a word
    // char so we don't match "5-year", "tax-free", "S-corp".
    { name: 'flag', re: /(?<!\w)--[a-z][a-z0-9_-]+/g },
    // Section sigil: § 199A, § 6651, § 1031, § 199A(b).
    { name: 'section', re: /§\s*\d+[A-Z]?(?:\([a-z0-9]+\))*/g },
    // IRS-style: Form 1040, Schedule C, Schedule E.
    { name: 'irsdoc', re: /\b(?:Form|Schedule)\s+[\dA-Z][\w-]*/g },
    // Retirement-plan / tax vocab (whole-word).
    { name: 'plan', re: /\b(?:401\(?k\)?|403\(?b\)?|457\(?b\)?|529|HSA|IRA|Roth|MAGI|AGI|RMD|SEP|SIMPLE)\b/g },
    // 1099 / W-2 / K-1 form codes with suffix.
    { name: 'formcode', re: /\b(?:1099-[A-Z]{1,4}|W-[124]|K-1|1040(?:-[A-Z]+)?|8606|8863|8829|8949|8960|4684|6251)\b/g },
    // FX pairs: 6 all-caps letters in a row.
    { name: 'fxpair', re: /\b[A-Z]{6}\b/g },
];

// Known terms always blacklisted regardless of frequency / pattern match.
// Curated; covers domain-specific labels the regex pass might miss.
const KNOWN_TERMS = new Set([
    // Brokers
    'E*TRADE', 'eTrade', 'Robinhood', 'Schwab', 'Charles Schwab',
    'Fidelity', 'Vanguard', 'Lightspeed', 'Webull', 'Tradier',
    'Interactive Brokers', 'IBKR', 'TD Ameritrade', 'Apex',
    'Tastytrade', 'tastyworks', 'SoFi', 'Wealthfront', 'Betterment',
    'Coinbase', 'Kraken', 'Binance', 'Ledger', 'Trezor',
    // Markets / exchanges / regulators
    'NYSE', 'NASDAQ', 'CBOE', 'AMEX', 'OTC',
    'FINRA', 'SEC', 'IRS', 'FDIC', 'SIPC', 'CFTC', 'IRC',
    // Products in our codebase
    'TraderView', 'Tesseract', 'PaddleOCR', 'Paddle',
    'Apple Vision', 'Vision Framework',
    'PostgreSQL', 'Postgres', 'SQLite', 'sqlx',
    'Tauri', 'OpenCV', 'ONNX', 'CTranslate2', 'LibreTranslate',
    'GitHub', 'macOS', 'iOS', 'Linux', 'Windows',
    'JavaScript', 'TypeScript', 'WebAssembly', 'Rust', 'Python',
    'Argos', 'OPUS-MT',
    'zpwr', 'zshrs', 'zinit', 'stryke',
    // Trading-jargon abbreviations frequently embedded in UI labels
    'P&L', 'PnL', 'OHLC', 'OHLCV', 'VWAP', 'TWAP', 'EMA', 'SMA', 'WMA',
    'RSI', 'MACD', 'ATR', 'ADX', 'CCI', 'OBV', 'MFI', 'PEAD',
    'BOP', 'BPV', 'IRR', 'CAGR', 'MFE', 'MAE', 'IV', 'GEX', 'OI',
    'PDT', 'YOLO', 'FOMO', 'YOY', 'MoM', 'QoQ', 'YTD', 'WoW',
    'KPI', 'AOV', 'CAC', 'LTV', 'CTR', 'CTC', 'ODC', 'AOTC', 'LLC',
    'NIIT', 'AMT', 'QBI', 'SSTB', 'NOL', 'AGI', 'MAGI', 'CTC', 'EITC',
    'FTF', 'FTP', 'FTD', 'IRC', 'TCJA', 'SECURE',
    // Tax forms & schedules people read as units
    'W-2', '1099-NEC', '1099-INT', '1099-DIV', '1099-MISC', '1099-B', '1099-K',
    'K-1', 'W-4', 'W-9', '1040', '1040-ES', '8949', '8606', '8863', '8829', '8960',
    'Form 1040', 'Schedule A', 'Schedule B', 'Schedule C', 'Schedule D',
    'Schedule E', 'Schedule SE',
]);

async function main() {
    const en = JSON.parse(await readFile(EN_PATH, 'utf8'));
    const seen = new Map(); // term → count

    for (const v of Object.values(en)) {
        if (typeof v !== 'string') continue;
        for (const { re } of PATTERNS) {
            const matches = v.match(re);
            if (!matches) continue;
            for (const m of matches) {
                const t = m.trim();
                if (!t) continue;
                if (STOP_WORDS.has(t.toUpperCase())) continue;
                // Drop 2-char unless it's a meaningful acronym (handled
                // by KNOWN_TERMS already).
                if (t.length < 2) continue;
                // Drop bare digits (would mask numerics — never desired).
                if (/^\d+$/.test(t) && t.length < 4) continue;
                seen.set(t, (seen.get(t) || 0) + 1);
            }
        }
    }

    // Always include curated KNOWN_TERMS even if regex didn't catch them.
    for (const t of KNOWN_TERMS) {
        if (!seen.has(t)) seen.set(t, 0);
    }

    // Sort: by length desc (so longer matches like "Charles Schwab"
    // bind before "Schwab"), then alpha. The translator masks in this
    // order so we don't accidentally mask "Schwab" inside "Charles
    // Schwab" and leave "Charles" unmasked.
    const sorted = [...seen.keys()].sort((a, b) => {
        if (b.length !== a.length) return b.length - a.length;
        return a.localeCompare(b);
    });

    const header =
        '# Proper-noun blacklist — terms the translator MUST NOT touch.\n' +
        '# Generated by i18n_extract_proper_nouns.mjs from app_i18n_en.json.\n' +
        '# Edit freely: add lines to force a term to stay verbatim, remove\n' +
        '# lines to let the translator have at it.\n' +
        '# Sorted: longest first (so multi-word matches win over substrings).\n' +
        '# One term per line. Blank lines and lines starting with # are ignored.\n' +
        '#\n' +
        `# Source EN keys: ${Object.keys(en).length}\n` +
        `# Total unique terms: ${sorted.length}\n` +
        '\n';

    await writeFile(OUT_PATH, header + sorted.join('\n') + '\n', 'utf8');
    console.log(`wrote ${sorted.length} terms → ${OUT_PATH}`);
    console.log('top 20 by frequency:');
    [...seen.entries()]
        .sort((a, b) => b[1] - a[1])
        .slice(0, 20)
        .forEach(([t, n]) => console.log(`  ${String(n).padStart(5)}  ${t}`));
}

main().catch((e) => {
    console.error(e);
    process.exit(1);
});
