#!/usr/bin/env python3
"""The token ledger: what each piece of work cost, on which tier, and whether it ran on the tier it should.

    python3 tools/tokens.py report                 this project's sessions: the main conversation by tier and day,
                                                   every agent with its kind of work, tier and tokens, and the advice
    python3 tools/tokens.py record [--stage S7]    add the agents not yet in results/token_ledger.csv to it (the
                                                   transcripts go with the container; the ledger is committed)
    python3 tools/tokens.py ledger [--stage S7]    the committed ledger summed by stage, kind and tier
    python3 tools/tokens.py checked AGENT ok|failed  mark an agent's work as checked by tools/verify_work.py

The transcripts are Claude Code's (~/.claude/projects/<this folder>/*.jsonl, or --transcripts DIR). The main
conversation's tokens are counted per reply (fresh input, cache writes, cache reads, output); an agent's
are the total its completion reports. Each kind of token is weighted by .claude/model_policy.toml
[token_weights] and each tier by its weight, so `weighted` ranks work by what it cost against the light tier.
An agent is "over" when it ran on a bigger tier than its kind of work needs; the advice says what moving
it down would have saved.

Copyright (c) 2026 Agastya. All rights reserved.
"""
import argparse
import csv
import json
import re
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

from common import ROOT

POLICY = ROOT / ".claude" / "model_policy.toml"
LEDGER = ROOT / "results" / "token_ledger.csv"
COLUMNS = ["agent", "date", "stage", "kind", "tier", "policy_tier", "tokens", "tool_uses", "minutes", "checked", "description"]
ORDER = ["light", "standard", "deep"]
STAGE = re.compile(r"\bS(\d{1,2})(?:[.\s]|\b)")


def policy(path=POLICY):
    return tomllib.loads(Path(path).read_text())


def tier_of(model, pol):
    """The tier a transcript's model name belongs to, or 'other'."""
    m = (model or "").lower()
    for name, t in pol["tiers"].items():
        if any(w in m for w in t["match"]):
            return name
    return "other"


def kind_of(description, pol):
    """The kind of work an agent's description says, first match in the policy's order, or 'unclassified'."""
    d = " " + (description or "").lower() + " "
    head = d.strip().split(":", 1)[0]
    for w in pol["work"]:
        if head == w["kind"]:
            return w["kind"], w["tier"]
    for w in pol["work"]:
        if any(word in d for word in w["words"]):
            return w["kind"], w["tier"]
    return "unclassified", ""


def stage_of(text):
    m = STAGE.search(text or "")
    return f"S{int(m.group(1))}" if m else ""


def weighted_usage(u, pol):
    """One reply's usage in fresh-input tokens (within its tier)."""
    w = pol["token_weights"]
    cc = u.get("cache_creation") or {}
    w1h = cc.get("ephemeral_1h_input_tokens", 0)
    w5m = cc.get("ephemeral_5m_input_tokens", u.get("cache_creation_input_tokens", 0) - w1h)
    return (u.get("input_tokens", 0) * w["input"] + w5m * w["cache_write_5m"] + w1h * w["cache_write_1h"]
            + u.get("cache_read_input_tokens", 0) * w["cache_read"] + u.get("output_tokens", 0) * w["output"])


def transcripts(folder=None):
    if folder:
        base = Path(folder)
    else:
        slug = re.sub(r"[^A-Za-z0-9]", "-", str(ROOT))
        base = Path.home() / ".claude" / "projects" / slug
    return sorted(base.glob("*.jsonl")) + sorted(base.glob("*/subagents/*.jsonl")) if base.exists() else []


NOTE = re.compile(r"<task-id>([^<]+)</task-id>.*?<usage><subagent_tokens>(\d+)</subagent_tokens>"
                  r"<tool_uses>(\d+)</tool_uses><duration_ms>(\d+)</duration_ms>", re.S)


def read(files, pol):
    """The main conversation's replies and the agents, from the transcripts."""
    main = []    # (date, tier, usage)
    agents = {}  # id -> record
    for f in files:
        sub = f.parent.name == "subagents"
        for line in f.read_text(encoding="utf-8", errors="replace").splitlines():
            try:
                d = json.loads(line)
            except ValueError:
                continue
            msg = d.get("message") if isinstance(d.get("message"), dict) else {}
            if d.get("type") == "assistant" and msg.get("usage") and not sub and not d.get("isSidechain"):
                if msg.get("model", "").startswith("<"):
                    continue
                main.append((d.get("timestamp", "")[:10], tier_of(msg.get("model"), pol), msg["usage"]))
            r = d.get("toolUseResult")
            if isinstance(r, dict) and r.get("agentId") and r.get("description"):
                a = agents.setdefault(r["agentId"], {"agent": r["agentId"], "tokens": 0, "tool_uses": 0, "minutes": 0.0})
                kind, want = kind_of(r["description"], pol)
                a.update(date=d.get("timestamp", "")[:10], description=r["description"], stage=stage_of(r["description"]),
                         kind=kind, policy_tier=want, tier=tier_of(r.get("resolvedModel"), pol))
            if "<task-notification>" not in line:
                continue
            found = []
            if d.get("type") == "user":
                found = NOTE.findall(json.dumps(msg).replace("\\n", "\n"))
            att = d.get("attachment") if isinstance(d.get("attachment"), dict) else {}
            if att.get("type") == "queued_command" and isinstance(att.get("usage"), dict):
                tid = re.search(r"<task-id>([^<]+)</task-id>", str(att.get("prompt", "")))
                u = att["usage"]
                if tid:
                    found.append((tid.group(1), u.get("totalTokens", 0), u.get("toolUses", 0), u.get("durationMs", 0)))
            for tid, tok, uses, ms in found:
                a = agents.setdefault(tid, {"agent": tid, "tokens": 0, "tool_uses": 0, "minutes": 0.0})
                # a notification repeats each time the agent stops; its counts are the agent's totals so far
                a["tokens"] = max(a["tokens"], int(tok))
                a["tool_uses"] = max(a["tool_uses"], int(uses))
                a["minutes"] = max(a["minutes"], round(int(ms) / 60000, 1))
    for a in agents.values():
        for k in COLUMNS:
            a.setdefault(k, "")
    return main, [a for a in agents.values() if a.get("description")]


def weight(tier, pol):
    return pol["tiers"].get(tier, {}).get("weight", max(t["weight"] for t in pol["tiers"].values()))


def advice(agents, pol):
    """Agents that ran above their kind's tier, and what the policy's tier would have cost."""
    rows, saved = [], 0.0
    for a in agents:
        want, got = a.get("policy_tier"), a.get("tier")
        if want in ORDER and got in ORDER and ORDER.index(got) > ORDER.index(want) and a["tokens"]:
            s = int(a["tokens"]) * (weight(got, pol) - weight(want, pol))
            saved += s
            rows.append((a, s))
    return rows, saved


def fmt(n):
    return f"{n/1e6:.2f} M" if n >= 1e6 else f"{n/1e3:.0f} k"


def cmd_report(a):
    pol = policy()
    main, agents = read(transcripts(a.transcripts), pol)
    print("Main conversation (fresh-input equivalent within its tier, then x tier weight):")
    by = defaultdict(lambda: [0, 0.0, 0])
    for day, tier, u in main:
        r = by[(day, tier)]
        r[0] += 1
        r[1] += weighted_usage(u, pol)
        r[2] += u.get("output_tokens", 0)
    total = 0.0
    for (day, tier), (n, w, out) in sorted(by.items()):
        total += w * weight(tier, pol)
        print(f"  {day}  {tier:<8} {n:5d} replies  {fmt(w):>9} weighted  {fmt(out):>8} output")
    print(f"  total weighted against the light tier: {fmt(total)}")
    lim, cold = pol["main"]["context_limit"], pol["main"]["cold_write"]
    ctx = [(u.get("cache_read_input_tokens", 0) + u.get("cache_creation_input_tokens", 0), u, tier) for _, tier, u in main]
    over = [c for c in ctx if c[0] > lim]
    wakes = [c for c in ctx if c[1].get("cache_creation_input_tokens", 0) > cold]
    if ctx:
        sizes = sorted(c[0] for c in ctx)
        print(f"  context per reply: median {fmt(sizes[len(sizes) // 2])}, largest {fmt(sizes[-1])}; "
              f"{len(over)} of {len(ctx)} replies above the limit of {fmt(lim)} (hand over to a new session sooner)")
        ww = sum(weighted_usage(u, pol) * weight(t, pol) for _, u, t in wakes)
        print(f"  {len(wakes)} cold wakes (more than {fmt(cold)} written to the cache in one reply): {fmt(ww)} weighted, "
              f"{100 * ww / total if total else 0:.0f} % of the main conversation (watch runs from a light agent, not here)")
    print(f"\nAgents ({len(agents)}):")
    for x in sorted(agents, key=lambda x: x["date"]):
        flag = "  OVER" if x["policy_tier"] in ORDER and x["tier"] in ORDER and ORDER.index(x["tier"]) > ORDER.index(x["policy_tier"]) else ""
        print(f"  {x['date']} {x['stage'] or '-':<4} {x['kind']:<12} ran {x['tier']:<8} wants {x['policy_tier'] or '?':<8} "
              f"{fmt(int(x['tokens'] or 0)):>8}  {x['description'][:48]}{flag}")
    rows, saved = advice(agents, pol)
    spent = sum(int(x["tokens"] or 0) * weight(x["tier"], pol) for x in agents)
    print(f"\nAgents' tokens x tier weight: {fmt(spent)}; on the policy's tiers they would have been {fmt(spent - saved)} "
          f"({100 * saved / spent if spent else 0:.0f} % less).")
    un = [x for x in agents if x["kind"] == "unclassified"]
    if un:
        print(f"{len(un)} agents' descriptions match no kind of work in the policy; give agents a description that says the work.")
    return 0


def load_ledger():
    if not LEDGER.exists():
        return []
    with LEDGER.open(newline="") as fh:
        return list(csv.DictReader(fh))


def save_ledger(rows):
    LEDGER.parent.mkdir(exist_ok=True)
    with LEDGER.open("w", newline="") as fh:
        w = csv.DictWriter(fh, COLUMNS, lineterminator="\n")
        w.writeheader()
        for r in sorted(rows, key=lambda r: (r["date"], r["agent"])):
            w.writerow({k: r.get(k, "") for k in COLUMNS})


def cmd_record(a):
    pol = policy()
    _, agents = read(transcripts(a.transcripts), pol)
    rows = load_ledger()
    have = {r["agent"]: r for r in rows}
    added = 0
    for x in agents:
        if a.stage and not x["stage"]:
            x["stage"] = a.stage
        old = have.get(x["agent"])
        if old:  # an agent resumed since: keep its check, take the larger totals
            for k in ("tokens", "tool_uses", "minutes"):
                if float(x[k] or 0) > float(old[k] or 0):
                    old[k] = x[k]
            continue
        rows.append(x)
        added += 1
    save_ledger(rows)
    print(f"tokens: {added} agents added, {len(rows)} in {LEDGER.relative_to(ROOT)}")
    return 0


def cmd_ledger(a):
    pol = policy()
    rows = [r for r in load_ledger() if not a.stage or r["stage"] == a.stage]
    sums = defaultdict(lambda: [0, 0, 0.0])
    for r in rows:
        s = sums[(r["stage"] or "-", r["kind"], r["tier"])]
        s[0] += 1
        s[1] += int(r["tokens"] or 0)
        s[2] += int(r["tokens"] or 0) * weight(r["tier"], pol)
    print(f"{'stage':<6}{'kind':<13}{'tier':<9}{'agents':>7}{'tokens':>11}{'weighted':>11}")
    for (st, k, t), (n, tok, w) in sorted(sums.items()):
        print(f"{st:<6}{k:<13}{t:<9}{n:>7}{fmt(tok):>11}{fmt(w):>11}")
    unchecked = [r for r in rows if r["checked"] not in ("ok", "failed")]
    failed = [r for r in rows if r["checked"] == "failed"]
    print(f"\n{len(rows)} agents; {len(rows) - len(unchecked)} checked ({len(failed)} failed), {len(unchecked)} not marked")
    _, saved = advice(rows, pol)
    if saved:
        print(f"over their tier: {fmt(saved)} weighted tokens would have been saved on the policy's tiers")
    return 0


def cmd_checked(a):
    rows = load_ledger()
    hit = [r for r in rows if r["agent"].startswith(a.agent)]
    if len(hit) != 1:
        print(f"tokens: {len(hit)} agents in the ledger start with {a.agent!r}", file=sys.stderr)
        return 1
    hit[0]["checked"] = a.verdict
    save_ledger(rows)
    print(f"tokens: {hit[0]['agent']} ({hit[0]['description']}) checked {a.verdict}")
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--transcripts", help="the folder of transcripts (default: this project's in ~/.claude/projects)")
    sp = ap.add_subparsers(dest="cmd", required=True)
    sp.add_parser("report").set_defaults(f=cmd_report)
    r = sp.add_parser("record")
    r.add_argument("--stage", help="the stage for agents whose description names none")
    r.set_defaults(f=cmd_record)
    g = sp.add_parser("ledger")
    g.add_argument("--stage")
    g.set_defaults(f=cmd_ledger)
    c = sp.add_parser("checked")
    c.add_argument("agent")
    c.add_argument("verdict", choices=["ok", "failed"])
    c.set_defaults(f=cmd_checked)
    a = ap.parse_args(argv)
    return a.f(a)


if __name__ == "__main__":
    sys.exit(main())
