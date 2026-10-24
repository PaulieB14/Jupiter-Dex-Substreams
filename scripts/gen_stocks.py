#!/usr/bin/env python3
"""Regenerate src/stocks_registry.rs from the official xStocks public API (Solana deployments)."""
import json, urllib.request
out, page = {}, 0
while True:
    url = f"https://api.xstocks.fi/api/v2/public/assets?network=Solana&pageSize=100&page={page}"
    d = json.load(urllib.request.urlopen(url, timeout=30))
    for n in d.get("nodes", []):
        for dep in n.get("deployments", []):
            if str(dep.get("network", "")).lower() == "solana" and dep.get("address"):
                u = (n.get("underlying") or {}).get("symbol") or n.get("underlyingSymbol") or ""
                out[dep["address"]] = (n["symbol"], u, n.get("isin") or "")
    if not d["page"]["hasNextPage"]:
        break
    page += 1
items = sorted(out.items())
with open("src/stocks_registry.rs", "w") as f:
    f.write("//! @generated from the official xStocks public API (https://api.xstocks.fi/api/v2/public/assets), Solana deployments.\n//! Regenerate with scripts/gen_stocks.py. Do not edit by hand.\n\n")
    f.write("/// (mint, xStock symbol, underlying ticker, ISIN), sorted by mint.\n")
    f.write(f"pub static XSTOCKS: [(&str, &str, &str, &str); {len(items)}] = [\n")
    for k, (s, u, i) in items:
        f.write(f'    ("{k}", "{s}", "{u}", "{i}"),\n')
    f.write("];\n")
print(len(items), "mints")
