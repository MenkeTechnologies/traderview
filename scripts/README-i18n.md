# App UI strings (`frontend/i18n/app_i18n_*.json`)

Mirror of the workflow used in `../audio_haxor/scripts/`.

## English catalog

- **Source of truth:** `frontend/i18n/app_i18n_en.json` (sorted keys).
- Add a key: edit `app_i18n_en.json` directly (keep keys sorted), then run
  `sync_locale_keys_from_en.py` so every other locale carries the key as
  a stub until the full machine-translation pass runs again.

## Other locales

26 shipped: `cs da de el es es_419 fi fr hi hu id it ja ko nb nl pl pt pt_br ro ru sv tr uk vi zh`.

At runtime `loadLocale()` (`frontend/js/i18n.js`) merges the active locale
on top of English, so a missing locale key falls back to English silently.
The locale JSONs still need the full key set so static parity checks pass
and translators have somewhere to write.

### Full machine translation (slow, needs network)

```bash
python3 -m venv .venv-i18n
.venv-i18n/bin/pip install deep-translator
.venv-i18n/bin/python scripts/gen_all_app_i18n_locales.py
```

Each per-locale generator (`scripts/gen_app_i18n_<loc>.py`) is a thin
wrapper around `_i18n_translate.translate_locale()` — translate each
unique English value once via Google Translate, map keys back, repair
`{tok}` placeholders that the translator capitalized, write sorted JSON.

Per-locale runs (subset of the above):

```bash
.venv-i18n/bin/python scripts/gen_app_i18n_de.py
.venv-i18n/bin/python scripts/gen_all_app_i18n_locales.py de fr es   # subset
```

### Fast (no network) stub sync

When you can't run the full pass yet:

```bash
python3 scripts/sync_locale_keys_from_en.py
```

For every non-English file: add missing keys with the English value as a
stub, drop keys English no longer has, keep existing translations for
surviving keys.

### `appFmt` placeholders (`{token}`)

Dynamic strings substitute English token names from JS callers
(`{symbol}`, `{count}`, etc.). Translated locales must keep the same
`{token}` spellings — `_i18n_translate.py` already repairs case-mangled
tokens after each Google Translate call, but new keys with unusual
spellings may still need manual touch-up.

### Timing

The English catalog is ~19k keys / ~14.7k unique values. At Google
Translate's anonymous-rate-limit ceiling (~0.06s sleep between calls), a
single locale takes ~15 minutes; the full 26-locale orchestrator takes
~5-6 hours wall-clock. Safe to interrupt and re-run.
