#!/usr/bin/env node
// i18n_revert_dnt.mjs — for every locale, restore DNT keys to the EN
// value. Fixes the proper-noun corruption introduced by mass
// translation passes that didn't filter brand names.

import { readFile, writeFile, rename, readdir } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { isDnt } from './i18n_dnt.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const I18N_DIR = resolve(__dirname, '../i18n');

async function main() {
    const en = JSON.parse(await readFile(resolve(I18N_DIR, 'app_i18n_en.json'), 'utf8'));
    const dntKeys = Object.keys(en).filter((k) => isDnt(k, en[k]));
    console.log(`DNT keys: ${dntKeys.length}`);

    const files = (await readdir(I18N_DIR))
        .filter((f) => /^app_i18n_.+\.json$/.test(f) && f !== 'app_i18n_en.json');

    let grandReverted = 0;
    for (const f of files) {
        const path = resolve(I18N_DIR, f);
        const locale = JSON.parse(await readFile(path, 'utf8'));
        let reverted = 0;
        for (const k of dntKeys) {
            if (locale[k] !== en[k]) {
                locale[k] = en[k];
                reverted += 1;
            }
        }
        if (reverted > 0) {
            const tmp = path + '.tmp';
            await writeFile(tmp, JSON.stringify(locale, null, 2) + '\n', 'utf8');
            await rename(tmp, path);
        }
        console.log(`${f.replace(/^app_i18n_|\.json$/g, '').padEnd(8)} reverted ${reverted}`);
        grandReverted += reverted;
    }
    console.log(`\nTotal reverted across all locales: ${grandReverted}`);
}

main().catch((e) => { console.error(e); process.exit(1); });
