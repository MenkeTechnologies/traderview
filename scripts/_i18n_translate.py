"""Shared engine for `gen_app_i18n_<locale>.py` per-locale generators.

Pattern mirrors `../audio_haxor/scripts/gen_app_i18n_*.py`:
- Read `frontend/i18n/app_i18n_en.json` as source of truth.
- Translate each *unique* English value once (Google Translate via
  `deep-translator`), then map keys back. Saves ~25% calls vs naive.
- Repair `{Token}` placeholders the translator capitalized.
- Write sorted, ensure_ascii=False, indent=2, trailing newline.
- Idempotent for a given English catalog; safe to interrupt and re-run
  (each call rebuilds the full output).

Requires: `.venv-i18n` with `pip install deep-translator`.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys
import time
from typing import Iterable

ROOT = pathlib.Path(__file__).resolve().parents[1]
I18N_DIR = ROOT / "frontend" / "i18n"
EN_PATH = I18N_DIR / "app_i18n_en.json"

_TOKEN_RE = re.compile(r"\{([A-Za-z_][A-Za-z0-9_]*)\}")


def _tokens(s: str) -> list[str]:
    return _TOKEN_RE.findall(s)


def _repair_placeholders(translated: str, expected: Iterable[str]) -> str:
    """Google Translate routinely capitalizes / spaces / drops `{tok}`s.

    Walk every expected token; if the translation has any case-variant of
    it inside braces, force it back to the original casing. Keeps the
    multiset parity that `i18n-per-key-placeholder-parity` enforces in
    audio-haxor (and that traderview's appFmt depends on)."""
    for tok in expected:
        if "{" + tok + "}" in translated:
            continue
        pat = re.compile(r"\{\s*" + re.escape(tok) + r"\s*\}", re.IGNORECASE)
        translated = pat.sub("{" + tok + "}", translated)
    return translated


def translate_locale(
    locale: str,
    google_target: str | None = None,
    *,
    sleep_s: float = 0.06,
    progress_every: int = 200,
) -> int:
    """Translate `app_i18n_en.json` → `app_i18n_<locale>.json`.

    Returns the number of keys written. Raises on import/IO failure;
    individual translate failures fall back to the English value so the
    catalog stays complete.
    """
    try:
        from deep_translator import GoogleTranslator
    except ImportError:
        print(
            "Install deep-translator: python3 -m venv .venv-i18n && "
            ".venv-i18n/bin/pip install deep-translator",
            file=sys.stderr,
        )
        raise SystemExit(1) from None

    target = google_target or locale
    out_path = I18N_DIR / f"app_i18n_{locale}.json"
    en: dict[str, str] = json.loads(EN_PATH.read_text(encoding="utf-8"))
    print(
        f"[{locale}] target={target} keys={len(en)} "
        f"unique_values={len(set(en.values()))}",
        file=sys.stderr,
        flush=True,
    )

    translator = GoogleTranslator(source="en", target=target)
    uniq_vals = list(dict.fromkeys(en.values()))
    val_map: dict[str, str] = {}
    t0 = time.time()
    for i, v in enumerate(uniq_vals):
        # Skip empty / whitespace / pure-punctuation strings; translator
        # raises on these and there is nothing to translate anyway.
        if not v.strip() or not re.search(r"[A-Za-z]", v):
            val_map[v] = v
        else:
            try:
                t = translator.translate(v)
                val_map[v] = t if isinstance(t, str) and t else v
            except Exception as e:  # noqa: BLE001 - network / API jitter
                if i < 3:
                    print(f"[{locale}] translate failed for {v!r}: {e}", file=sys.stderr)
                val_map[v] = v
        if (i + 1) % progress_every == 0:
            elapsed = time.time() - t0
            rate = (i + 1) / elapsed if elapsed > 0 else 0.0
            eta = (len(uniq_vals) - (i + 1)) / rate if rate > 0 else 0.0
            print(
                f"[{locale}] {i + 1}/{len(uniq_vals)} "
                f"elapsed={elapsed:.0f}s rate={rate:.1f}/s eta={eta:.0f}s",
                file=sys.stderr,
                flush=True,
            )
        if sleep_s:
            time.sleep(sleep_s)

    out: dict[str, str] = {}
    for k, v in en.items():
        tx = val_map.get(v, v)
        expected = _tokens(v)
        if expected:
            tx = _repair_placeholders(tx, expected)
        out[k] = tx

    out_path.write_text(
        json.dumps(out, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"[{locale}] wrote {len(out)} keys → {out_path}", file=sys.stderr)
    return len(out)
