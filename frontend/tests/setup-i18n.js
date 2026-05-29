// Preload the English i18n catalog so helper modules that resolve labels
// via t() get real strings during unit tests instead of raw keys.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { setMap } from '../js/i18n.js';

const here = dirname(fileURLToPath(import.meta.url));
const enPath = resolve(here, '..', 'i18n', 'app_i18n_en.json');
setMap(JSON.parse(readFileSync(enPath, 'utf8')));
