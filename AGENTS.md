# AGENTS.md

This repository is **agent-driven**, but with strict safeguards:

1) **You MUST build the repo first** before doing any meaningful work.
2) **You MUST use the freshly-built local binary** to fetch/generate any docs/specs/info the agent needs.
3) **Do not fetch external documents** (web browsing, curl/wget, etc.) unless explicitly allowed by a human.

If you are an automated coding agent, follow this document strictly.

---

## 0. Golden Rule: Build First

Before you:
- read or interpret project behavior,
- suggest changes,
- edit code,
- run tests,
- open a PR,

you MUST perform a clean build and confirm it succeeded.
