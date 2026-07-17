# Tool description guides the model toward keyword queries

The `web_search` tool's schema description and `_TOOL_DESCRIPTION` (in `src/agent_web_search/server.py`) read as explicit instructions to form `search_query` as concise keywords rather than a full natural-language question, with before/after examples and a list of filler words to drop (EN + CN). This looks like prescriptive prompting inside a tool schema. This ADR records why it is there, so a future contributor trimming "verbose" descriptions does not silently undo a measured relevance fix.

## Context

After the v0.2.0 release, a user reported that broad, colloquial queries returned mostly irrelevant results — the canonical example was `今天有什么科技新闻` ("what tech news is there today") returning BBC World Cup coverage, VPN ads, and gambling pages instead of any tech news.

Diagnosis (recorded against the spec, issue #1) traced this to the query form, not to the Source or to tokenization: the colloquial filler (`今天有什么` — "today has what") gets tokenized into high-frequency tokens that dominate ranking and crowd out the actual intent (`科技新闻`). The same intent expressed as keywords (`科技新闻 今日 2026年7月`) returns sharply better results. English queries are far less affected because English filler (`how do I`, `what is`) tokenizes more gently and the upstream Source (via `ddgs`) handles English questions reasonably well.

## Considered options

- **Source-side query rewrite (normalize every query to keywords before sending)** — rejected. Adds a transformation layer, a new failure surface, and a maintenance burden (rewrite rules per language). It also duplicates work the model is already good at: given a clear instruction, the model will emit keywords on its own. The fix belongs at the prompt boundary, not in the search pipeline.

- **Switch or add Sources to get better question handling** — rejected. The diagnosis showed the Source already handles keyword queries well; the problem was upstream of the Source, in the query the model chose to send. Changing Sources would not address the root cause and would trade away the stability properties established in ADR-0006.

- **Guide the model via the tool description (chosen)** — the `search_query` parameter description and `_TOOL_DESCRIPTION` tell the model to prefer concise keywords, give a before/after example, and name the filler words to drop in both English and Chinese. Zero logic change; only string constants moved.

## Why it's acceptable

The change touches no code path — only two string constants in `server.py`. There is no stability risk: the orchestration, fan-out, extraction, and assembly layers are untouched. The only behavioral effect is on what the model chooses to put in `search_query`, which is exactly the layer where the root cause lived.

Empirical verification (v0.2.1 live test, 2026-07-18) confirmed the effect on the originally-failing case:

| Query form | Query | Result |
| --- | --- | --- |
| Full question | `今天有什么科技新闻` | Almost entirely off-target — BBC World Cup, DeepSeek/Hangzhou commentary, VPN, gambling |
| Keywords | `科技新闻 今日 2026年7月` | 9/10 on-target — IT之家, 雪球, 东方财富, 网易科技日历, 新华网 AI 大会 |

A matched pair on an English technical query (`how do I use tokio spawn in rust`) returned high-quality results in both forms, confirming the diagnosis that the problem is concentrated in colloquial, high-filler queries — exactly the case the description steers the model away from.

## Consequences

- The `search_query` description and `_TOOL_DESCRIPTION` in `server.py` must not be "cleaned up" into terser forms without re-running the keyword-vs-question comparison. The instruction text is the fix.
- If a future Source change introduces one that handles natural-language queries well (e.g. an LLM-mediated Source), the guidance can be relaxed — but that decision belongs to a new ADR, not to a casual edit.
- This is a prompt-bound mitigation, not a guarantee. A model that ignores the description and sends a full question anyway will still get the poorer results; the description raises the probability of the right query form, it does not enforce it.
