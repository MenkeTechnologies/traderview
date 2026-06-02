#!/usr/bin/env python3
"""Build frontend/i18n/app_i18n_fi.json from app_i18n_en.json.

Requires: .venv-i18n with deep-translator (see scripts/README-i18n.md).
"""
from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _i18n_translate import translate_locale  # noqa: E402


if __name__ == "__main__":
    translate_locale("fi", google_target="fi")
