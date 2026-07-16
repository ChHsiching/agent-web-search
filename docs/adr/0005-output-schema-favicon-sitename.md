# Output schema: favicon and site_name are derived, zero-dependency

The target tool (`web_search_prime`) returns per result: title, url, summary, **site name**, and **favicon**. SearXNG's JSON does not provide a site name or favicon per result, so both must be derived. This ADR records how, so the output schema is fixed before the extract/output module is written.

## Considered options

- **Fetch favicon via a third-party service (e.g. `https://www.google.com/s2/favicons?domain=...`)** — rejected: even though we'd only construct the URL (not fetch it), it is a soft dependency on a third-party endpoint, which conflicts with the dependency-minimization stance (same reason Brave/Playwright were rejected in ADR-0001). Also unnecessary for our consumers.
- **Omit favicon / site_name fields** — rejected: breaks drop-in compatibility. The tool must be a 1:1 replacement for `web_search_prime`, so the fields must be present in the output.
- **Derive both from the result's own URL (chosen)**:
  - `favicon` = `{scheme}://{host}/favicon.ico`, constructed from the result URL. Points at the source site's own favicon; no third-party service, no network call by us, zero external dependency.
  - `site_name` = derived from the URL hostname (strip leading `www.`, take the registrable-domain label).

## Why it's acceptable

The actual consumers of this tool are coding agents (Claude Code, Codex, ZCode). They ingest search results as **text in their context** and do not render icons or site names. The favicon field is therefore functionally inert for the real use case — its only job is to exist so the schema matches `web_search_prime`. Constructing a URL string with zero fetching and zero third-party dependency satisfies that at no cost.

## Consequences

- The output struct is fixed: `{ title, url, summary (extract), site_name, favicon }` — a 1:1 match with the target tool's advertised fields.
- No extra dependency, no extra network call, no extra failure surface.
- If a future consumer needs real favicon images, they fetch `{scheme}://{host}/favicon.ico` themselves; that is not our concern.
