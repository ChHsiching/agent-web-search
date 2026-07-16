# Content extraction library: readabilityrs

Page-body extraction (the Extract field, controlled by `content_size`) uses the **`readabilityrs`** crate — a Rust port of Mozilla's modern Readability algorithm (the one behind Firefox's Reader View).

## Considered options

- **`readability` (kumabook)** — rejected despite being the most-downloaded readability crate (~1.16M). It is a port of the **arc90 Readability project (2009)**, not Mozilla's modern algorithm. Last published 2023-12; stale. High download count reflects "first to claim the name," not quality.
- **`readabilityrs` (theiskaa)** — chosen. Port of Mozilla's modern Readability. Actively maintained: v0.1.0 → v0.1.3 across Nov 2025 – Apr 2026, ~134k downloads on the latest version and rising, maintainer merging bugfix/refactor PRs as recently as Apr 2026.
- **`llm_readability` (spider-rs)** — rejected. Also a Mozilla port and actively updated, but at v0.0.17 (unstable API) and an order of magnitude fewer downloads. Name is attractive ("for LLM") but the version signal says the API is not settled.
- **Hand-written heuristics** — rejected: main-content detection varies heavily per site template; hand-rolling it is the high-maintenance path we explicitly want to avoid.
- **External reader service (e.g. the target system's web_reader, Jina)** — rejected: couples us to an external dependency and failure point, contradicting independent/free/stable.

## Consequences

- Extract quality rides on a battle-tested algorithm (Mozilla Readability), not bespoke heuristics.
- It is a pure algorithm crate, not a heavy runtime dependency — single-binary distribution is preserved.
- We inherit `readabilityrs`'s maintenance: if it goes stale we may need to fork, but the fan-out/extract layer is isolated behind our own trait so swapping implementations is local.
