# OODA (any repo)

**Location:** `~/OODA.md`  
**Scope:** Any codebase.  
**Purpose:** One rotation: **Observe → Decide → Act → Lock → Ship**.

**Always load:** `~/AXIOMS.md`.  
**When claims need hostility:** `~/PROBE.md`.  
**Product:** `ROOT/DESIGN.md` (each repo should have one).  
**Product toolkit (optional):** `ROOT/TOOLS.md` — never replaces home process files.

---

## Always on (from AXIOMS — summary)

- First principles + enough planning to act  
- E-M, honesty/fail-closed, security posture, reverse entropy  
- Immune tests, **hygiene**, line pressure ≤256 (or repo lock)  
- Power law: Decide ≤ **5**  

Full text: **`~/AXIOMS.md`**.

---

## Adapter (fill every session)

```markdown
ROOT:
Product binaries / how to build & run:
Backlog file (or issues):
Handoff file (e.g. PROGRESS.md):
Must-not-regress rails (discover under ROOT):
Hygiene commands (line lock, fmt, lint — if any):
ROOT/DESIGN.md present?:
ROOT/TOOLS.md present? (product-only):
```

---

## 1. Observe

1. Handoff (PROGRESS / notes)  
2. Backlog  
3. **Tree health / hygiene snapshot** — line lock if present; obvious cruft; red rails  
4. Product TOOLS metrics if `ROOT/TOOLS.md` defines them  
5. Security-sensitive surfaces (auth, secrets, I/O, exec) if this product has them  

Ask: best next **product** step (E-M / information value)?  
Plan only enough to Act (AXIOMS: first principles + planning).

---

## 2. Decide (≤5)

Prefer, in order:

1. **Red product rails** / regressions  
2. **Honesty / security** fail-open holes  
3. **Next backlog item** (DESIGN-aligned)  
4. **DESIGN gap** → add to backlog, then do  
5. **Entropy / hygiene / split** only if blocking or cheap  

---

## 3. Act

- Implement on the **real** product path for this ROOT  
- Fail-closed unfinished work (or explicit residual)  
- Stay under line pressure or split same rotation  
- Ship tests with behavior when claims matter  
- Respect security posture (default-deny privileges; no secrets in tree)  

---

## 4. Lock (hygiene + rails)

**Hygiene is required here** (AXIOMS §1.7), not deferred to PROBE:

1. **Line lock** — repo script or AXIOMS default (≤256)  
2. **Cruft** — no accidental binaries, caches, debug dumps, empty pointer docs  
3. **Secrets** — none in diff or committed files  
4. **Temps** — traps/cleanup for build-run artifacts this turn touched  
5. **Docs honesty** — no new half-truths; residual named if unfinished  
6. **Product rails** — only scripts/CI this ROOT actually has (discover; don’t invent)  
7. Backlog checkboxes only if **really** done (fixtures + product path)  

If hygiene fails → do not Ship; fix or residual.

---

## 5. Ship

- Commit/push as appropriate (no secrets; no force-push of shared main unless owner policy)  
- Short report: what shipped, design/backlog area, **\(S\)** / **\(O\)** if used, hygiene OK  
- Prefer **reverse entropy** (trust surface cleaner or flat with reason)  
- **Never** auto-claim release/beta unless owner policy already did  

---

## When to run PROBE

Use **`~/PROBE.md`** in the same ROOT when:

- Security or privilege claims need falsification  
- You suspect fail-open, torn state, or synthetic green rails  
- After a large change to boundaries (I/O, auth, exec, caps)  

PROBE does **not** replace Lock hygiene; it **adds** hostility.

---

## Master prompt

```text
Run OODA from ~/OODA.md with ~/AXIOMS.md loaded.
Target ONE repository ROOT. Fill the adapter.
ROOT/DESIGN.md and ROOT/TOOLS.md are product inputs only if present.
Observe → Decide (≤5) → Act → Lock (hygiene + rails) → Ship.
First principles, E-M, fail-closed, reverse entropy, security posture.
No soft-pass. No foreign product assumptions.
For adversarial assumption hunt: ~/PROBE.md.
```

---

## Product session notes (optional)

A short product pointer (e.g. `loop - openOODA.md`) may set ROOT, DESIGN, backlog, and preferred rails only.  
Do **not** fork the whole OODA file per product.

---

*Process kit: **OODA** · **AXIOMS** · **PROBE**.*
