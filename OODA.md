# OODA (any repo)

**Location:** `~/OODA.md`  
**Scope:** Any codebase.  
**Purpose:** One rotation: **Observe → Decide → Act → Lock → Ship**.

**Always load:** `~/RULES.md`.  
**When claims need hostility:** `~/PROBE.md`.  
**Product:** `ROOT/DESIGN.md` (each repo should have one).  
---

## Always on (from RULES — summary)

- First principles + enough planning to act  
- E-M, honesty/fail-closed, security posture, reverse entropy  
- Immune tests, **hygiene**, line pressure ≤256 (or repo lock)  
- Power law: Decide ≤ **5**  

Full text: **`~/RULES.md`**.

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
```

---

## 1. Observe

1. Handoff (PROGRESS / notes)  
2. Backlog  
3. **Tree health / hygiene snapshot** — line lock if present; obvious cruft; red rails  
4. Security-sensitive surfaces (auth, secrets, I/O, exec) if this product has them  

Ask: best next **product** step (E-M / information value)?  
Plan only enough to Act (RULES: first principles + planning).

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

**Hygiene is required here** (RULES §1.7), not deferred to PROBE:

1. **Execute Hygiene Check** — strictly follow the 6 steps from `RULES.md §1.7` (Line pressure, Cruft, Secrets, Temps, Git, Docs).
2. **Product rails** — only scripts/CI this ROOT actually has (discover; don't invent).
3. Backlog checkboxes only if **really** done (fixtures + product path).

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

## Swarm operations (bundled from SWARM.md)

These rules apply to **any** LLM-driven session, single- or multi-agent.
They are the default unless an operator explicitly overrides them per call.

### Model tiering (right-sizing intelligence)
Never default to heavy reasoning models for lightweight tasks. Match the
cognitive load to the engine.

| Tier | Examples | Allowed use cases |
|------|----------|-------------------|
| **Fast / Light** | Gemini Flash, GPT-4o-mini, Grok-fast, Minimax-light | Hygiene audits, line-lock checkers, grammar enforcement, simple bash, deterministic formatting. |
| **Heavy / Pro** | Gemini Pro, GPT-4o / o1, Grok-2, Minimax-pro | Deep architectural reasoning, compiler emission, interprocedural state, complex multi-file logic bugs. |

### Small ships, fast fail (task granularity)
Do not hand agents monolithic missions. Massive tasks force the LLM to
pull huge context into memory, inflating token cost on every action.

- **Surgical scoping** — slice tasks into single-file, localized prompts.
- **Fail-fast** — stop and report back after the goal or first failure;
  do not enter long autonomous trial-and-error loops.

### Ban redundant swarms (the debugger fallacy)
If an agent gets stuck (infinite compile loop, deadlock, hallucination):

- **DO NOT** spawn additional heavy subagents to "debug" the first one —
  multiplies token bleed.
- **DO** use fast, localized deterministic probes (`grep`, `tail`,
  direct bash) to diagnose the root cause yourself.
- **DO** ruthly kill stuck background processes / agents to prevent
  context-window runaway.

### Architectural isolation (workspace strategy)
Concurrent engineering agents must modify **strictly orthogonal files**.
If domains overlap, force them into isolated sandboxes, `/tmp` dirs, or
separate git branches. Prevents file-lock collisions, `/tmp` races,
corrupted ASTs.

### QA & adversarial probes
QA / Boundary agents focus entirely on edge cases (mutual recursion
limits, taint propagation drops, macro explosions) to rapidly falsify
the happy-path claims of the primary engineers. On a fail-open hole,
the QA agent must enforce a fail-closed trap (`process_exit(1)`) and
halt, not autonomously attempt a large refactor.

---

## Master prompt

```text
Run OODA from ~/OODA.md with ~/RULES.md loaded.
Target ONE repository ROOT. Fill the adapter.
ROOT/DESIGN.md is a product input only if present.
Observe → Decide (≤5) → Act → Lock (hygiene + rails) → Ship.
First principles, E-M, fail-closed, reverse entropy, security posture.
No soft-pass. No foreign product assumptions.
Swarm ops (tiering, scoping, isolation, adversarial QA) are bundled above.
For adversarial assumption hunt: ~/PROBE.md.
```

---

## Product session notes (optional)

A short product pointer (e.g. `loop - openOODA.md`) may set ROOT, DESIGN, backlog, and preferred rails only.  
Do **not** fork the whole OODA file per product.

---

*Process kit: **OODA** · **RULES** · **PROBE**. Swarm ops bundled.*
