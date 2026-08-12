# SWARM.md — Swarm Operations & Token Efficiency

**Location:** `~/SWARM.md`  
**Scope:** Any multi-agent or LLM-driven coding environment (Gemini, Grok, OpenAI, Minimax, etc.).  
**Purpose:** Agnostic first principles for orchestrating LLMs and agentic swarms, strictly minimizing token waste, and preventing runaway compute costs.

---

## 1. Model Tiering (Right-Sizing Intelligence)
Never default to heavy reasoning models for lightweight tasks. Match the cognitive load to the engine.

| Tier | Ecosystem Examples | Allowed Use Cases |
|------|--------------------|-------------------|
| **Fast / Light** | Gemini Flash, GPT-4o-mini, Grok-fast, Minimax-light | Hygiene audits, line lock checkers (`check_file_lines.sh`), grammar enforcement (ASD-STE100), simple bash script execution, and deterministic formatting. |
| **Heavy / Pro** | Gemini Pro, GPT-4o / o1, Grok-2, Minimax-pro | Deep architectural reasoning, compiler emission logic, interprocedural state tracking, and resolving complex multi-file logic bugs. |

---

## 2. Small Ships, Fast Fail (Task Granularity)
Do not hand agents monolithic missions (e.g., "Build a full backend"). Massive tasks force the LLM to pull huge context windows into memory, drastically inflating the token cost of every single action and reasoning loop.

- **Surgical Scoping:** Slice tasks into single-file, highly localized prompts.
- **Fail-Fast:** Instruct the LLM to stop and report back immediately after hitting its specific goal or first failure, rather than entering long, autonomous trial-and-error loops in the background.

---

## 3. Ban Redundant Swarms (The Debugger Fallacy)
If an agent gets stuck (e.g., triggers an infinite compilation loop, deadlocks a terminal, or enters a hallucination loop):
- **DO NOT** spawn additional heavy subagents to "debug" the first agent. This multiplies token bleed exponentially.
- **DO** use fast, localized deterministic probes (`grep`, `tail`, direct bash commands) to diagnose the root cause yourself.
- **DO** ruthlessly kill rogue or stuck background processes / agents using your platform's process management tools to prevent context-window runaway.

---

## 4. Architectural Isolation (Workspace Strategy)
When spinning up concurrent engineering agents, ensure they are modifying **strictly orthogonal files**.
- If domains overlap (e.g., two agents modifying the same AST parser or build script), force them into isolated sandboxes, temporary directories, or separate git branches.
- This prevents file lock collisions, `/tmp` race conditions, and corrupted ASTs.

---

## 5. QA & Adversarial Probes
- QA / Boundary agents should focus entirely on edge cases (mutual recursion limits, taint propagation drops, macro explosions) to rapidly falsify the 'happy path' claims of the primary engineers.
- If a fail-open hole is found, the QA agent should immediately enforce a fail-closed trap (e.g., `process_exit(1)`) and halt, rather than autonomously attempting a massive refactor of the underlying feature.
