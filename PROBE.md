# Code Probe (any repo)

**Location:** `~/PROBE.md`  
**Scope:** Any codebase.  
**Purpose:** Adversarial **assumption hunt** — falsify claims. Not a compliance audit product. Not green-CI theater.

**Always load:** `~/AXIOMS.md` (bar). **Build rotation:** `~/OODA.md`.  
**Product docs** are **inputs** only.

**Hygiene:** Required every Ship via **OODA Lock** (AXIOMS §1.7). PROBE **adds** hostility (security, lifecycle, chaos). It does not replace Lock.

---

## First principle

**Software stability is an illusion maintained only until an assumption is violated.**

A probe does not “verify the code works.” It **destroys assumptions** behind code, docs, and green rails: fail-open paths, torn states, synthetic tests, lifecycle and security debt.

---

## 0. Defaults

| Constant | Default | Notes |
|----------|---------|--------|
| **MAX_LINES** | **256** | Prefer repo lock if present |
| **Hang budget** | 5–10s per hostile input | Hang = **FAIL** |
| **CRIT dual-repro** | 2 independent runs | Never close CRIT on one witness |

---

## 0.1 Product inputs (ROOT only)

| Artifact | Use |
|----------|-----|
| `DESIGN.md` | What “aligned” means |
| Line lock / fmt / lint | Hygiene + \(O\) |
| Smokes / CI | **Suspects**, not proof |
| Residual lists | Claim honesty |

```bash
ROOT=<repo>
test -f "$ROOT/DESIGN.md" && echo HAS_DESIGN=1
test -x "$ROOT/scripts/check_file_lines.sh" && "$ROOT/scripts/check_file_lines.sh"
```

**Hard rule:** Smokes never skip boundary/chaos dual-repro.

---

## 1. Roles (adversarial independence)

| Role | May | Must not |
|------|-----|----------|
| **Orchestrator** | Schedule; **re-run** CRITs; write report | Implement fixes; rubber-stamp |
| **Worker** | Fix + fault-inject on **product** binaries | Self-certify CRIT |
| **Challenger** | Falsify Worker claims; reject synthetic tests | Ship fix same turn as finding |

One agent may wear all three roles — still **re-run** CRITs independently (second shell / clean state).

---

## 2. Boundary pack (bottom-up)

Start at OS/network/auth edges. Trace **backwards**. Find **torn states**.

| # | Attack | Pass |
|---|--------|------|
| B1 | Happy path with full privilege | Succeeds as claimed |
| B2 | Missing privilege / auth | Fail-closed |
| B3 | Forged / zero / wrong credential on **product path** | Deny |
| B4 | OS fail after open (full disk, short write) | Error, not silent Ok |
| B5 | Forbidden path / permission deny | No ambient success |

---

## 3. Chaos pack

| # | Attack | Pass |
|---|--------|------|
| C1 | FD starve (`ulimit -n 32`/`64`) | No hang |
| C2 | RO / unwritable temp | Non-zero |
| C3 | Concurrent valid + garbage | No deadlock |
| C4 | Empty / truncated / binary / oversized | Fail-closed; hang = FAIL |
| C5 | Missing runtime / bad link | Fail-closed |

---

## 4. Lifecycle pack

| # | Check |
|---|--------|
| L1 | Handles closed on all paths |
| L2 | Partial write/read = failure where integrity claimed |
| L3 | Children reaped |
| L4 | Temps cleaned on success **and** failure |
| L5 | No secrets in world-readable fixed temps |
| L6 | Question large clones if zero-copy claimed |

---

## 5. Hygiene + maintainability (with AXIOMS)

Confirm AXIOMS hygiene (line lock, cruft, secrets, temps, claim/docs match) — same bar as OODA Lock.  
PROBE **extends** with hostile cases (oversized input, RO temp, secret-in-log attempts).

| # | Check |
|---|--------|
| E1 | Line lock green (or ratchet) |
| E2 | Lock misses owned types (e.g. scripts lock ignores) |
| E3 | Ceiling list (~90% MAX_LINES) |

---

## 6. Claim–evidence pack

| # | Check |
|---|--------|
| K1 | Docs vs product path |
| K2 | CI/smokes vs real entrypoints |
| K3 | “Not supported” is non-zero / explicit error |
| K4 | Build-path matrix (test × build × release × flags) |
| K5 | Security claims ≤ what rails prove |

---

## 7. Anti-synthetic (Challenger)

Reject if any **no**:

1. Same binary users get?  
2. Same product path/flags?  
3. Exit codes without pipe-masked `$?`?  
4. Would fail if the bug still existed?  
5. Attack one layer below the claim?  
6. Auth/caps: missing / wrong / forge covered?

```bash
cmd >out 2>err
echo exit=$?
```

---

## 8. Phases

| Phase | Action |
|-------|--------|
| **P0** | Adapter + product inputs + surface list |
| **P1** | Boundary |
| **P2** | Build matrix |
| **P3** | Chaos |
| **P4** | Lifecycle |
| **P5** | Claims / security honesty |
| **P6** | Hygiene + line pressure |
| **P7** | Challenger re-runs Worker fixes |
| **P8** | Report |

**Done when:** dual-repro CRITs, clean exits measured correctly, findings complete, fixes re-challenged.

---

## 9. Finding schema

```markdown
## Finding F##
Severity: CRIT | HIGH | MED | LOW
Assumption destroyed:
Surface:
Repro (shell):
Actual output / exit:
Would smoke catch? (yes/no + why):
Residual after fix? (none | text):
```

| Severity | Meaning |
|----------|---------|
| **CRIT** | Bypass or wrong success on product path |
| **HIGH** | Fail-open, hang, silent wrong behavior |
| **MED** | Lifecycle / incomplete seal / hygiene debt |
| **LOW** | Docs drift, ceiling pressure |

```markdown
## Session
ROOT:
HAS_DESIGN:
Hygiene / line lock:
Smokes (suspects):
```

---

## 10. Adapter

```markdown
## Adapter: <project>
Root path:
Product binaries / entrypoints:
Build & run:
Trust boundaries:
Owned source globs:
MAX_LINES (default 256):
Line lock / hygiene commands:
ROOT/DESIGN.md?:
Known residual claims:
CI / smoke commands (suspects):
Threat model one-liner:
```

---

## 11. Master prompt

```text
Initialize Code Probe from ~/PROBE.md with ~/AXIOMS.md loaded.
Target ONE ROOT. Fill the adapter.
Product DESIGN.md is an input only if present; CI / scripts/ bundles ship the gates.
Roles: Orchestrator, Worker, Challenger (or one agent wearing all three with dual-repro).
P0→P8: boundary, chaos, lifecycle, claims, hygiene.
CRIT: dual independent repro; no pipe-masked exit codes.
Real product binaries only. Reject synthetic tests.
Not a compliance audit — assumption hunt / code review.
```

---

## 12. Not this framework

- Not regulatory/compliance “audit” product  
- Not “run smokes and write PASSED”  
- Not a substitute for OODA Lock hygiene on every ship  
- Not permission to rewrite without dual-repro  

---

*Process kit: **PROBE** · **AXIOMS** · **OODA**.*
