#!/usr/bin/env python3
"""Refresh the lensisku dictionary headword corpus used by tests/dictionary_corpus.rs.

Sources (public, no auth; see https://lensisku.lojban.org/swagger-ui/openapi.json):
  1. GET /api/export/cached/en/json
     The cached English dictionary export (best positive-scored definition per word).
  2. GET /api/jbovlaste/definitions?languages=2&page=N&per_page=1000
     Every English definition regardless of score. The full uncached export
     (/api/export/dictionary/en?positive_scores_only=false) needs a bearer token,
     so zero/negative-scored words (e.g. many "obsolete fu'ivla" such as
     `matriocka`) are collected from this paginated listing instead.

Output: tests/data/lensisku-en-words.tsv.gz with columns
  word<TAB>word_type<TAB>in_cached_export(0/1)
sorted by word, one row per distinct headword.
"""
import gzip
import json
import os
import sys
import time
import urllib.request

BASE = "https://lensisku.lojban.org/api"
OUT = os.path.join(os.path.dirname(__file__), "..", "tests", "data", "lensisku-en-words.tsv.gz")
UA = {"User-Agent": "camxes-rs corpus fetcher"}


def get_json(url):
    for attempt in range(5):
        try:
            req = urllib.request.Request(url, headers=UA)
            with urllib.request.urlopen(req, timeout=300) as r:
                return json.load(r)
        except Exception as e:  # noqa: BLE001
            print(f"retry {attempt + 1} for {url}: {e}", file=sys.stderr)
            time.sleep(2 * (attempt + 1))
    raise SystemExit(f"failed: {url}")


def main():
    words = {}  # word -> [type, in_cached]
    cached = get_json(f"{BASE}/export/cached/en/json")
    for e in cached:
        words[e["word"]] = [e["word_type"], 1]
    print(f"cached export: {len(cached)} entries", file=sys.stderr)

    page, seen, total = 1, set(), None
    while True:
        d = get_json(f"{BASE}/jbovlaste/definitions?languages=2&page={page}&per_page=1000")
        total = d["total"]
        defs = d["definitions"]
        if not defs:
            break
        for x in defs:
            seen.add(x["definitionid"])
            w = x["valsiword"]
            if w not in words:
                words[w] = [x["type_name"], 0]
        print(f"page {page}: {len(seen)}/{total}", file=sys.stderr)
        if len(seen) >= total:
            break
        page += 1
    if len(seen) < total:
        print(f"warning: only {len(seen)} of {total} definitions seen", file=sys.stderr)

    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with gzip.open(OUT, "wt", encoding="utf-8", compresslevel=9) as f:
        f.write("word\tword_type\tin_cached_export\n")
        for w in sorted(words):
            t, c = words[w]
            w1 = w.replace("\t", " ").replace("\n", " ")
            f.write(f"{w1}\t{t}\t{c}\n")
    print(f"wrote {len(words)} headwords to {OUT}", file=sys.stderr)


if __name__ == "__main__":
    main()
