# Axioms (any repo)

**Location:** `~/AXIOMS.md`  
**Scope:** Any codebase.  
**Purpose:** Always-on **first principles** of how we work. Not product scripts. Not DESIGN.

**Siblings:** `~/OODA.md` (rotation) · `~/PROBE.md` (assumption hunt).  
**Product docs** (`ROOT/DESIGN.md`, `ROOT/TOOLS.md`) are **inputs** — they never replace this file.

---

## 0. Where things run

| Axiom (here) | Executed in |
|--------------|-------------|
| Hygiene requirements | **OODA Lock** (every ship); **PROBE** when security-shaped |
| Security posture | **Always** in Act; **PROBE** to falsify |
| First principles + planning | **OODA Observe / Decide** |
| E-M | **OODA** every turn |
| Reverse entropy | **OODA Ship** report; **PROBE** claim pack |
| Line pressure | **OODA Lock**; **PROBE** E-pack |
| Immune tests | **OODA Act**; **PROBE** rejects synthetic rails |

---

## 1. Core axioms (non-negotiable)

### 1.1 First principles
- Prefer root causes and mechanisms over slogans and checklist theater.  
- If a claim is not grounded in behavior + evidence, treat it as untested (\(U\)).  
- **Plan just enough** to act: clear goal, constraints, and done-bar — then implement. Planning without Act violates E-M.

### 1.2 E-M (energy / act)
Every turn: **rank**, then **implement** or record an explicit fail-closed residual.  
No observe-only theater. No “we should someday” without a backlog entry or residual note.

### 1.3 Honesty / fail-closed
Unfinished work fails closed or is named residual.  
No soft-pass, no silent OK, no fake “done.”

### 1.4 Security (default posture)
- Prefer **default-deny** for privileged effects (auth, caps, secrets, network, exec).  
- Privileges must be **explicit** on the product path (params, tokens, roles) — not ambient accident.  
- Do not claim security stronger than rails prove.  
- Secrets stay out of logs, fixed world-readable temps, and commits.  
- **PROBE** is how we challenge security claims; axioms are the bar.

### 1.5 Reverse entropy
The **trust surface** should get cleaner over time, not dirtier.  
Prefer ships that **lower** disorder \(S\) (or hold \(S\) with an honest one-line why).  
Raising \(S\) (new untested claims, fail-open holes, hand-waves, oversize) is a bad ship unless the residual is explicit.

### 1.6 Immune tests
Behavior that is claimed ships with **pass and fail** proof where it matters.  
Green happy-path only is not immunity.

### 1.7 Hygiene (required)
Hygiene is an **axiom**, not optional polish. Minimum bar before Ship (OODA Lock):

| Check | Intent |
|-------|--------|
| **Line pressure** | Owned sources ≤ **256** lines (or repo lock); split at functional seams |
| **No cruft** | No debug dumps, accidental binaries, `__pycache__`, or “pointless pointer” docs in the tree |
| **Secrets** | No credentials/tokens in source or commit |
| **Temps** | Build/run temps cleaned on success **and** failure (traps / equivalent) |
| **Git** | No force-push of shared main unless owner policy; don’t commit secrets or generated trash |
| **Claims match tree** | Docs don’t assert features the code doesn’t have (or residual is named) |

Product may add hygiene scripts in `ROOT/TOOLS.md` or CI; absence of a script does **not** waive the axiom.

### 1.8 Short feedback
Prefer small changes that produce a **true** pass/fail signal quickly.

---

## 2. Heuristics (support the axioms)

| Name | Rule |
|------|------|
| **Power law** | Decide at most **5** items this turn; ignore the long tail. |
| **Line pressure** | Default **≤256** lines / owned file (or repo lock). |
| **Entropy \(S\)** (optional score) | \(S = U + F + W + O\) |

| Term | Meaning |
|------|---------|
| **\(U\)** | Untested claims |
| **\(F\)** | Fail-open / silent OK |
| **\(W\)** | Hand-waves (stub as done) |
| **\(O\)** | Oversize owned files |

Report when used: `S: n (Δ …) — U=_ F=_ W=_ O=_`  
Repos may extend \(S\) in **ROOT/TOOLS.md** (product-only terms).

---

## 3. Science as *source* (motivation, not a catalog)

| Domain | Idea | Axiom / place |
|--------|------|----------------|
| Control / OODA | Short loops | OODA.md |
| Flight / energy | Act, don’t only plan | E-M |
| Thermo | Trust-surface disorder | Reverse entropy / \(S\) |
| Pareto / math | Few items dominate | Power law ≤5 |
| Biology | Attack *and* defend | Immune tests |
| Chemistry | No fake purity | Fail-closed |
| Materials | Complexity limits | Line pressure / hygiene |

---

## 4. Using product docs (combine, don’t replace)

| Input | Role |
|-------|------|
| **`ROOT/DESIGN.md`** | What this product is. Don’t edit DESIGN for fake victory. |
| **Backlog** | What to build after red rails / honesty. |
| **`ROOT/PROGRESS.md`** | Handoff; update on Ship. |
| **`ROOT/TOOLS.md`** | Product-only metrics/scripts — **additional** constraints. |
| **Rails / CI** | Lock suspects; PROBE still challenges claims. |

---

## 5. Ranking (default Decide order)

1. Red product rails / regressions  
2. Honesty / security fail-open holes  
3. Next backlog item (DESIGN-aligned)  
4. DESIGN gap → backlog, then do  
5. Entropy / hygiene / split only if blocking or cheap  

---

## 6. Non-goals

No language mandating, no product binary lists, no smoke inventories. Those are **ROOT** docs.

---

*Process kit: **AXIOMS** · **OODA** · **PROBE**.*
