"""Fails when a @tr() text in ui/ has no translation in one of the .po catalogs.

Usage: python3 -I scripts/check-translations.py [ui dir] [translations dir]
"""
import re
import sys
from pathlib import Path

ui_dir = Path(sys.argv[1] if len(sys.argv) > 1 else "ui")
translations_dir = Path(sys.argv[2] if len(sys.argv) > 2 else "translations")


def unquote(text):
    return text.encode().decode("unicode_escape").encode("latin-1").decode("utf-8")


used = set()
for file in ui_dir.rglob("*.slint"):
    used.update(unquote(t) for t in re.findall(r'@tr\("((?:[^"\\]|\\.)*)"', file.read_text()))

failed = False
for catalog in sorted(translations_dir.glob("*/LC_MESSAGES/*.po")):
    pairs = re.findall(r'^msgid "((?:[^"\\]|\\.)*)"\nmsgstr "((?:[^"\\]|\\.)*)"', catalog.read_text(), re.M)
    translated = {unquote(msgid) for msgid, msgstr in pairs if msgid and msgstr}
    missing = sorted(used - translated)
    if missing:
        failed = True
        print(f"{catalog}: missing {len(missing)}:")
        for text in missing:
            print(f"  {text}")
    else:
        print(f"{catalog}: all {len(used)} texts translated")
sys.exit(1 if failed else 0)
