#!/usr/bin/env node
// i18n_translate.mjs — pump untranslated i18n keys through a local
// LibreTranslate instance.
//
// Spin up the server first (port 5050 because macOS AirPlay Receiver
// squats on 5000):
//   docker run -d --name libretranslate -p 5050:5000 \
//     libretranslate/libretranslate \
//     --load-only en,de,es,fr,it,nl,pt,ja,ko,zh,hi,id,pl,ru,tr,uk,vi,ar,cs,da,el,fi,hu,nb,ro,sv
//
// Then:
//   node scripts/i18n_translate.mjs                              # all locales
//   node scripts/i18n_translate.mjs --locale de,fr,es            # subset
//   node scripts/i18n_translate.mjs --refresh-leaking            # also retranslate values still equal to EN
//   node scripts/i18n_translate.mjs --limit 200                  # only first N missing keys
//   node scripts/i18n_translate.mjs --batch 50                   # batch size per HTTP request
//   node scripts/i18n_translate.mjs --concurrency 4              # parallel HTTP requests per locale
//   node scripts/i18n_translate.mjs --endpoint http://host:5000  # override server URL
//   node scripts/i18n_translate.mjs --api-key KEY                # hosted instance auth
//   node scripts/i18n_translate.mjs --dry-run                    # count only, no HTTP
//
// `{tok}`-style interpolation placeholders are protected by swapping
// them to curly-brace sentinels (`{xtvi0}`, `{xtvi1}`, …) that LT
// translation and restored after. A translation that drops the
// sentinel is rejected (caller keeps the EN value).

import { readFile, writeFile, rename } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { isDnt, maskProperNouns } from './i18n_dnt.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, '..');
const EN_PATH = resolve(ROOT, 'i18n/app_i18n_en.json');

// Mapping from our locale code → LibreTranslate language identifier.
// Locales not in this map are skipped.
const LOCALE_TO_LT = {
    ar: 'ar',
    cs: 'cs',
    da: 'da',
    de: 'de',
    el: 'el',
    es: 'es',
    es_419: 'es',
    fi: 'fi',
    fr: 'fr',
    hi: 'hi',
    hu: 'hu',
    id: 'id',
    it: 'it',
    ja: 'ja',
    ko: 'ko',
    nb: 'nb',
    nl: 'nl',
    pl: 'pl',
    pt: 'pt',
    pt_br: 'pt-BR',
    ro: 'ro',
    ru: 'ru',
    sv: 'sv',
    tr: 'tr',
    uk: 'uk',
    vi: 'vi',
    zh: 'zh-Hans',
};

// ── CLI parsing ────────────────────────────────────────────────────────────

const args = parseArgs(process.argv.slice(2));

function parseArgs(argv) {
    const out = {
        locale: null,
        refreshLeaking: false,
        limit: Infinity,
        // Big batches + sequential are dramatically faster on LibreTranslate's
        // CPU-bound Argos backend than small concurrent ones. ~30 strings/sec
        // at batch=300 conc=1 on default Docker Desktop. Tested with
        // LT_THREADS=8 LT_BATCH_LIMIT=400 LT_REQ_TIME_LIMIT=600 on the server.
        batch: 300,
        concurrency: 1,
        endpoint: process.env.LIBRETRANSLATE_URL ?? 'http://localhost:5050',
        apiKey: process.env.LIBRETRANSLATE_API_KEY ?? '',
        dryRun: false,
    };
    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        const eat = (name) => {
            if (a === `--${name}`) return argv[++i];
            if (a.startsWith(`--${name}=`)) return a.slice(`--${name}=`.length);
            return null;
        };
        let v;
        if ((v = eat('locale')) !== null) out.locale = v.split(',');
        else if (a === '--refresh-leaking') out.refreshLeaking = true;
        else if (a === '--dry-run') out.dryRun = true;
        else if ((v = eat('limit')) !== null) out.limit = Number(v);
        else if ((v = eat('batch')) !== null) out.batch = Number(v);
        else if ((v = eat('concurrency')) !== null) out.concurrency = Number(v);
        else if ((v = eat('endpoint')) !== null) out.endpoint = v;
        else if ((v = eat('api-key')) !== null) out.apiKey = v;
        else {
            console.error(`unknown arg: ${a}`);
            process.exit(64);
        }
    }
    return out;
}

// ── Placeholder protection ─────────────────────────────────────────────────

const TOKEN_RE = /\{[A-Za-z0-9_]+\}/g;

function protect(value) {
    // 1) Mask `{token}` interpolation placeholders.
    const map = new Map();
    const back = new Map();
    let next = 0;
    let masked = value.replace(TOKEN_RE, (m) => {
        if (map.has(m)) return map.get(m);
        const sentinel = `{xtvi${next++}}`;
        map.set(m, sentinel);
        back.set(sentinel, m);
        return sentinel;
    });
    // 2) Mask proper nouns from the curated blacklist
    // (Fidelity, E*TRADE, Tesseract, FINRA, …). Different sentinel
    // prefix (`__PN…__`) so the two namespaces don't collide.
    const pn = maskProperNouns(masked);
    masked = pn.masked;
    for (const [sentinel, term] of pn.back) back.set(sentinel, term);
    return { masked, back };
}

function restore(translated, back) {
    if (back.size === 0) return translated;
    let out = translated;
    for (const [sentinel, token] of back) {
        if (!out.includes(sentinel)) return null;
        out = out.split(sentinel).join(token);
    }
    return out;
}

// ── Skip rules ─────────────────────────────────────────────────────────────

function shouldTranslate(value) {
    if (typeof value !== 'string') return false;
    if (value.trim().length < 2) return false;
    const stripped = value.replace(TOKEN_RE, '').trim();
    if (stripped.length < 2) return false;
    if (!/[A-Za-z]{2,}/.test(stripped)) return false;
    return true;
}

// ── LibreTranslate HTTP ───────────────────────────────────────────────────

async function libretranslateBatch(items, source, target, opts) {
    // LibreTranslate accepts `q` as an array; returns `translatedText` as
    // a matching-length array.
    const body = {
        q: items.map((it) => it.value),
        source,
        target,
        format: 'text',
    };
    if (opts.apiKey) body.api_key = opts.apiKey;

    // 5-min per-batch hard timeout — if a LibreTranslate worker hangs
    // (we've seen this on cold language pairs), bail and let the caller
    // retry instead of waiting forever on the TCP socket.
    const ac = new AbortController();
    const timer = setTimeout(() => ac.abort(), 5 * 60 * 1000);
    let resp;
    try {
        resp = await fetch(`${opts.endpoint}/translate`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
            signal: ac.signal,
        });
    } finally {
        clearTimeout(timer);
    }
    if (!resp.ok) {
        const txt = await resp.text();
        throw new Error(`HTTP ${resp.status}: ${txt.slice(0, 300)}`);
    }
    const json = await resp.json();
    const translated = json.translatedText;
    if (!Array.isArray(translated) || translated.length !== items.length) {
        throw new Error(`LibreTranslate returned malformed payload (expected array of ${items.length})`);
    }
    return items.map((it, i) => ({ key: it.key, value: translated[i] }));
}

// ── Per-locale work ───────────────────────────────────────────────────────

async function loadJson(path) {
    return JSON.parse(await readFile(path, 'utf8'));
}

async function writeJsonAtomic(path, obj) {
    const tmp = path + '.tmp';
    const serialized = JSON.stringify(obj, null, 2) + '\n';
    await writeFile(tmp, serialized, 'utf8');
    await rename(tmp, path);
}

async function pumpLocale(localeCode, ltLang, en, opts) {
    const localePath = resolve(ROOT, `i18n/app_i18n_${localeCode}.json`);
    let locale;
    try {
        locale = await loadJson(localePath);
    } catch (e) {
        if (e.code === 'ENOENT') {
            console.log(`${localeCode}: no app_i18n_${localeCode}.json — skipping`);
            return { translated: 0, skipped: 0, kept: 0 };
        }
        throw e;
    }

    const todoKeys = [];
    for (const key of Object.keys(en)) {
        const enVal = en[key];
        if (!shouldTranslate(enVal)) continue;
        // Proper-noun gate: brand names, product names, broker dropdowns,
        // CLI flags — never translate these (they're identical across
        // every locale and machine translation actively corrupts them,
        // e.g. "Fidelity" → "Treue", "E*TRADE" → "E*HANDEL").
        if (isDnt(key, enVal)) continue;
        const cur = locale[key];
        const missing = cur === undefined || cur === null || cur === '';
        const leaking = !missing && cur === enVal;
        if (missing || (opts.refreshLeaking && leaking)) {
            todoKeys.push(key);
            if (todoKeys.length >= opts.limit) break;
        }
    }

    if (todoKeys.length === 0) {
        console.log(`${localeCode}: nothing to translate`);
        return { translated: 0, skipped: 0, kept: 0 };
    }

    console.log(`${localeCode}: ${todoKeys.length} keys to translate (lt=${ltLang})…`);
    if (opts.dryRun) return { translated: 0, skipped: 0, kept: todoKeys.length };

    // Slice into batches; run `concurrency` batches in parallel; persist
    // after each completed window so progress survives a crash.
    const batches = [];
    for (let i = 0; i < todoKeys.length; i += opts.batch) {
        batches.push(todoKeys.slice(i, i + opts.batch));
    }

    let translated = 0;
    let skipped = 0;
    let done = 0;

    for (let i = 0; i < batches.length; i += opts.concurrency) {
        const window = batches.slice(i, i + opts.concurrency);
        const results = await Promise.allSettled(window.map((chunk) => translateChunk(chunk, ltLang, en, opts)));
        for (const r of results) {
            if (r.status === 'rejected') {
                console.error(`${localeCode}: batch failed: ${r.reason?.message ?? r.reason}`);
                continue;
            }
            for (const out of r.value.applied) locale[out.key] = out.value;
            translated += r.value.applied.length;
            skipped += r.value.skipped;
        }
        done += window.reduce((acc, c) => acc + c.length, 0);
        await writeJsonAtomic(localePath, locale);
        process.stdout.write(`  ${localeCode}: ${Math.min(done, todoKeys.length)}/${todoKeys.length}\r`);
    }

    console.log(`\n${localeCode}: +${translated} translated, ${skipped} skipped`);
    return { translated, skipped, kept: 0 };
}

async function translateChunk(chunk, ltLang, en, opts) {
    const items = [];
    const backs = new Map();
    for (const key of chunk) {
        const { masked, back } = protect(en[key]);
        items.push({ key, value: masked });
        backs.set(key, back);
    }
    const translated = await libretranslateBatch(items, 'en', ltLang, opts);
    const applied = [];
    let skipped = 0;
    for (const t of translated) {
        const final = restore(t.value, backs.get(t.key));
        if (final == null) {
            skipped += 1;
            continue;
        }
        applied.push({ key: t.key, value: final });
    }
    return { applied, skipped };
}

// ── main ──────────────────────────────────────────────────────────────────

async function main() {
    // Sanity-ping the server so we fail fast with a clear message.
    try {
        const resp = await fetch(`${args.endpoint}/languages`);
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    } catch (e) {
        console.error(`Can't reach LibreTranslate at ${args.endpoint}: ${e.message}`);
        console.error('Spin one up with:');
        console.error('  pnpm i18n:server');
        console.error('  # or: docker run -d --name libretranslate -p 5050:5000 libretranslate/libretranslate');
        process.exit(1);
    }

    const en = await loadJson(EN_PATH);
    const allLocales = Object.keys(LOCALE_TO_LT);
    const wantLocales = args.locale ?? allLocales;

    const unsupported = wantLocales.filter((l) => !(l in LOCALE_TO_LT));
    if (unsupported.length) {
        console.error(`skipping unsupported locales: ${unsupported.join(', ')}`);
        console.error('(supported: ' + allLocales.join(', ') + ')');
    }

    const grand = { translated: 0, skipped: 0, kept: 0 };
    for (const loc of wantLocales) {
        if (!(loc in LOCALE_TO_LT)) continue;
        const res = await pumpLocale(loc, LOCALE_TO_LT[loc], en, args);
        grand.translated += res.translated;
        grand.skipped += res.skipped;
        grand.kept += res.kept;
    }
    console.log(`\nGRAND TOTAL: +${grand.translated} translated, ${grand.skipped} skipped, ${grand.kept} would-translate`);
}

main().catch((e) => {
    console.error(e);
    process.exit(1);
});
