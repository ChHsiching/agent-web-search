# 调研笔记：agent-web-fetch 最近的文档更新（新手引导为主）

- 调研日期：2026-10-31
- 调研对象：本地仓库 `C:\Users\Administrator\Git\agent-web-fetch`（远端 ChHsiching/agent-web-fetch，分支 main，HEAD = `8dcd263`，与 origin/main 同步）
- 调研方式：只读。全部结论来自本地 git 历史（`git log` / `git show`）与当前文件内容，未做任何改动。
- 报告位置说明：本仓库（agent-web-search）的 `docs/` 下目前只有 `adr/` 与 `agents/` 两个子目录（约定见 `AGENTS.md` 与 `docs/agents/domain.md`），没有既有的笔记目录约定，因此按任务指示新建 `docs/notes/` 存放本报告。

## 1. 近 30 条提交中的文档相关提交

`git log --date=short -30`（2026-10-27 至 2026-10-31，中间无其他提交）中，标题带 `Docs:` 或新增文档文件的提交共 7 条：

| Commit | 日期 | 说明 | 涉及文档 |
| --- | --- | --- | --- |
| `b089a5b` | 2026-10-27 | raw-text passthrough 决策记录（CONTEXT.md 术语、ADR-0006、研究笔记） | `CONTEXT.md`、`docs/adr/0006-raw-text-passthrough.md`（新增 30 行）、`docs/research/issue-11-plain-text-support.md`（新增 254 行） |
| `0ad341b` | 2026-10-27 | 在工具描述、两个 README、错误注释中呈现 raw-text passthrough | `README.md`、`README.zh-CN.md`（另含 `internal/mcpserver/server.go`、`internal/fetch/errors.go`） |
| `ec34a87` | 2026-10-31 | **README 新手引导（onboarding）重构：快速开始、用法、对比**（本次调研核心） | `README.md`（+318 行改）、`README.zh-CN.md`（+210 行改）、`docs/research/readme-onboarding-patterns.md`（新增 354 行） |
| `9b08245` | 2026-10-31 | 两个 README 顶部加 release / license 徽章 | `README.md`、`README.zh-CN.md` |
| `1073331` | 2026-10-31 | size-cap 决策记录、max_mb 参数行、Size Cap 术语 | `docs/adr/0007-size-cap.md`（新增 27 行）、`CONTEXT.md`、`README.md`、`README.zh-CN.md` |
| `553fc85` | 2026-10-31 | 修正 return_format 在工具 schema 和两个 README 中的措辞 | `README.md`、`README.zh-CN.md`、`internal/mcpserver/server.go` |
| `1d43a70` | 2026-10-31 | 全仓清理"叙述性注释"（风格类 Docs 提交） | 不涉及 README/docs/；只改 `.github/workflows/*`、`Makefile`、`build.sh`、`internal/**` 中的代码注释 |

另有一条风格配套提交 `085e540`（2026-10-31，"drop em dashes from new comments per repo style"）同样只动代码注释。README 的"去破折号"风格化实际是在 `ec34a87` 里一并完成的（见其提交说明 "Removes em-dash punctuation throughout"）。

更早的 README 历史（供背景参考，2026-07 的一波新手引导基础工作）：`f3f818e`（跨平台构建+安装文档，#8）、`f8a8c61`（Windows 安装细节、配置文件位置、验证步骤）、`e47680e`（安装章节改写为客户端无关）、`58ef6e8`（server 改名 chhsich-web-fetch）、`6d12efd`（新增简体中文 README）、`e42d4c2`/`f609307`/`c82084f`（安装位置与措辞修正）、`7feff01`（2026-09-03，更新 LICENSE 和 README）。本轮 10-31 的重构是在这批基础上的重新组织。

## 2. 核心提交 `ec34a87`：README 新手引导重构逐条摘要

提交说明原文（`git show ec34a87`）：

> Patterns researched from high-star MCP tools (see docs/research/readme-onboarding-patterns.md): 3-step quick start and details-collapsed per-client configs (browser-tools-mcp, official fetch server), comparison table and FAQ (wigolo), go install channel (fzf). Adds Usage section showing raw-text passthrough, Troubleshooting, and Contributing. Removes em-dash punctuation throughout; fixes stale Go 1.22 requirement (go.mod is 1.26).

### 2.1 README.md（英文）新增/重组的章节与内容

来源：`git show ec34a87 -- README.md`；现行文件 `C:\Users\Administrator\Git\agent-web-fetch\README.md`。

1. **新增 `## Why this instead of a paid web-reader`（现 L14）**：把原有三个卖点（free/uncapped、reliable launch、near-zero maintenance）重命名收敛成显式"替代付费方案"章节，并新增收尾定位段：
   > "Hosted web-reader services move your URLs and page content through a vendor cloud and meter every request. This tool does the same job on your machine, in a process your MCP client already manages, for $0 per page."

2. **`## Install` 改为 `## Quick start`（现 L72），三步编号结构**：
   - 步骤 1 标题从 "Download the binary" 改为 "**Install** the binary for your platform"（现 L74）。发布版二进制表格仍是首选（零前置依赖是差异点），另新增两个折叠块：
     - `<details> Prefer go install (requires Go 1.26+)`：`go install github.com/ChHsiching/agent-web-fetch/cmd/agent-web-fetch@latest`，并说明二进制落在 `$GOBIN` / `$GOPATH/bin`，Windows 通常为 `%USERPROFILE%\go\bin`。
     - `<details> Build from source`：原独立 H2 "Build from source" 降级折叠进来，同时把过期的 "Requires Go 1.22+" 修正为 "Go 1.26+ (matches go.mod)"。
   - 步骤 2 "Register it with your MCP client"（现 L131）：
     - 新增新手常见误区说明，**"You never launch it yourself"**：手动运行二进制没有任何作用，stdio MCP server 由客户端拉起，并链接官方 transports 说明（91k 星仓库先例）。
     - 各客户端配置从平铺改为 `<details>` 折叠：原有 ZCode、Claude Code 两条，**新增 Zed**（`context_servers` 键、无 `type` 字段、UI 添加路径、settings.json 位置）和**新增 opencode**（`mcp` 键、`"type": "local"`、`command` 为数组、`opencode.json` 位置）两条。
   - 步骤 3 从 "Verify it works" 升级为 "**Use it**"（现 L249），合并原步骤 4（可选日志）：
     - 按客户端给出注册状态检查方法：Zed 看 Settings → AI → MCP Servers 的绿点；`opencode mcp list`；`claude mcp list`。
     - 给出可直接粘贴的示例提示词，且**点名 server**以避免命中客户端内置 fetch：
       > "Use the chhsich-web-fetch fetch tool to read https://example.com"
     - 新增疑难场景 "**If the agent keeps using its own built-in fetch**"：Zed / opencode 内置网页工具不会因注册而被禁用，模型继续选内置工具"看起来和工具没生效一模一样"。两种解法：提示词/`AGENTS.md` 里点名 server；或关闭内置工具（Zed 建 agent profile 关内置 `fetch`；opencode 在 `opencode.json` 用 `"tools": { "webfetch": false }`）。

3. **新增 `## Usage`（现 L281）**：面向略读者的真实示例：
   - raw-text 透传示例：用工具抓自己的 README（`raw.githubusercontent.com/.../README.md`），展示逐字节返回、不加代码围栏；JSON（package.json、API 响应、GeoJSON）同样透传。
   - 普通网页走提取；带参数示例 `fetch({ "url": ..., "timeout": "45s", "no_cache": true })`。
   - "提示词说法 → 参数"映射清单（如"重新抓一次，别用缓存" → `no_cache: true`；"这个网站很慢" → 更大 `timeout`）。

4. **新增 `## Comparison`（现 L311）**：与托管式 web-reader MCP（Firecrawl / Jina Reader 一类）的 6 行对比表（API key、每页成本、内容处理位置、运行时、JS 渲染、许可证），并带时效脚注："Feature standing as of October 2026. Check each service's current pricing."

5. **新增 `## Troubleshooting`（现 L325）**：原"可选启用文件日志"小节收编于此，给出 `WEB_FETCH_LOG` 环境变量写法（含按 server 注入 env 的 JSON 示例）；另含两个 FAQ 折叠块："Free? What's the catch?" 与 "Can I use this at work?"（AGPL 双许可答问）。

6. **新增 `## Contributing`（现 L372）**：欢迎 issue/PR 及有效缺陷报告要素（URL + 期望 + 实际）；本地开发命令（`go test ./...`、`go vet ./...`、`./build.sh`）；给 AI agent/新人的仓库导览一句（`CONTEXT.md` 术语表、`docs/adr/` 决策、`AGENTS.md` 约定）；提交 PR 即同意贡献条款的声明。

7. **风格与细节**：全篇去 em dash（与 `085e540` 的代码注释风格一致）；工具参数表中 `url` 默认值从"—"改为 "none"；开头定位句点名具体客户端（Claude Code、ZCode）。

### 2.2 README.zh-CN.md（中文）同步镜像

来源：`git show ec34a87 -- README.zh-CN.md`。中文版逐节对应新增：`## 为什么用它，而不是付费 web-reader`（L10）、`## 快速开始`（L48，含 `go install` / 从源码构建两个折叠块、Zed / opencode 折叠配置）、`### 3. 用起来`（L192，含各客户端状态检查、"如果 agent 一直用它自带的 fetch"）、`## 用法`（L213）、`## 对比`（L236）、`## 故障排查`（L249，含两条 FAQ 折叠）、`## 参与贡献`（L278）。两份 README 结构完全对齐。

### 2.3 配套研究文档 `docs/research/readme-onboarding-patterns.md`（新增，354 行）

来源：`git show ec34a87:docs/research/readme-onboarding-patterns.md`；现行文件 `C:\Users\Administrator\Git\agent-web-fetch\docs\research\readme-onboarding-patterns.md`。

这是一份"先调研、后改稿"的一手研究笔记，方法为抓取 8 个高星同类仓库 README 原文（2026-10-31 经 raw.githubusercontent.com 获取，星数取自 GitHub API）：

| 研究对象 | 星数 | 借鉴点 |
| --- | --- | --- |
| modelcontextprotocol/servers（root + src/fetch/） | 91,706 | "stdio server 由客户端启动、自己跑会挂住是正常的"解释；`<details>` 折叠各安装法 |
| junegunn/fzf | 83,876 | 单二进制卖点写法；按安装渠道分小节 |
| firecrawl/firecrawl-mcp-server | 7,729 | 按客户端分小节 + 精确 JSON |
| AgentDeskAI/browser-tools-mcp | 7,341 | **"Why this instead of X" + 三步编号快速开始**（被选为主骨架，Pattern C） |
| KnockOutEZ/wigolo | 5,508 | 对比表 + FAQ + 时效脚注 |
| goreleaser/goreleaser | 16,127 | 反例：README 不宜把安装外包给不存在的 docs 站 |
| jae-jae/fetcher-mcp | 1,087 | 中英切换行的惯例 |

其结论（Verdict）原文要点：以 browser-tools-mcp 的"紧凑叙事"为骨架，借用官方 fetch server 的 `<details>` 折叠、wigolo 的对比表与 FAQ、fzf 的分渠道安装；指出当时 README 真正缺的三件事：`go install` 渠道（含 Go 版本要求）、展示 raw-text 透传的 Usage 章节、承载 free/uncapped 定位的紧凑对比表。

### 2.4 其余文档提交摘要

- `9b08245`（README.md L4-5 / README.zh-CN.md L4-5）：顶部新增两枚 shields 徽章——GitHub release（`img.shields.io/github/v/tag/...?include_prereleases&sort=semver`，链接最新 release）与 License（链接 LICENSE 文件）。
- `1073331`：新增 ADR `docs/adr/0007-size-cap.md`（核心决策："reject, never truncate"，默认 2 MB、`max_mb` 可提到硬上限 8 MB，超限报错并注明实际大小与上限；依据是模型上下文规模）；`CONTEXT.md` 新增 **Size Cap** 术语（含 `_Avoid_` 反义词）；两个 README 的工具参数表各加一行 `max_mb`（README.md L56 / README.zh-CN.md L36）。
- `553fc85`：`return_format` 参数描述改为如实表述（README.md L55 / README.zh-CN.md L35）：HTML 页面用 `text` 返回不带 markdown 语法的纯文本；纯文本文件两种取值内容相同。同步改 `internal/mcpserver/server.go` 的 tool schema 措辞。
- `0ad341b`（2026-10-27）：README "What it does" 新增 raw-text 透传段落（text/*、JSON、XML、YAML、JS 跳过提取、UTF-8 原样返回）；"How it works" 流程图新增透传分支；中文版同步。
- `b089a5b`（2026-10-27）：`CONTEXT.md` 新增 **Passthrough** 术语；新增 ADR-0006 与研究笔记 `docs/research/issue-11-plain-text-support.md`（issue #11 的调研设计，无行为变更）。

## 3. agent-web-fetch 当前 README.md 章节目录

现行 `README.md`（HEAD `8dcd263`）结构，行号来自当前文件：

```
1    # agent-web-fetch（语言切换 + release/license 徽章）
14   ## Why this instead of a paid web-reader
29   ## What it does
51     ### Tool parameters
61     ### What it does *not* do
72   ## Quick start
74     ### 1. Install the binary for your platform
131    ### 2. Register it with your MCP client
249    ### 3. Use it
281  ## Usage
311  ## Comparison
325  ## Troubleshooting
354  ## How it works
372  ## Contributing
393  ## License
400    ### Contribution Terms
```

`README.zh-CN.md` 同构：为什么用它 / 它做什么（工具参数、它不做什么）/ 快速开始（下载二进制、注册、用起来）/ 用法 / 对比 / 故障排查 / 工作原理 / 参与贡献 / 许可证（贡献条款）。

## 4. 文档更新伴随的配套改动清单

- **docs/ 目录新布局**：本轮在原有 `docs/adr/0001..0005`、`docs/agents/`（domain / issue-tracker / triage-labels）基础上，新增了 `docs/research/` 目录（两份研究笔记：`issue-11-plain-text-support.md`、`readme-onboarding-patterns.md`）和 `docs/adr/0006-raw-text-passthrough.md`、`docs/adr/0007-size-cap.md`。即：**README 改版前先落一份调研笔记，决策先落 ADR，README 只呈现结论**。
- **CONTEXT.md**：新增 Passthrough（`b089a5b`）与 Size Cap（`1073331`）两个术语条目，均含 `_Avoid_` 反义提示。
- **工具 schema 与 README 同步**：`internal/mcpserver/server.go` 中 tool description / 参数 jsonschema 描述与 README 参数表多次同步更新（`0ad341b`、`4ec9985`、`553fc85`）。
- **无独立 INSTALL 文件**：安装内容全部收在 README Quick start 内（含 `go install`、源码构建两个折叠块），未拆分 INSTALL 文档，也无配置示例文件变更。
- **代码注释风格统一**：`1d43a70`、`085e540` 两条 Docs 提交清理全仓叙述性注释与 em dash（只动代码注释，不动 README）。

## 5. agent-web-fetch 新手引导内容清单（供逐项对照）

以下为 `ec34a87`（及配套提交）后 README 中面向新手/小白的全部引导要素，可逐项对照 agent-web-search 的 README 现状：

对照于 2026-10-31 由主会话完成，依据为 agent-web-search 的 `README.md`、`README.zh-CN.md`、`pyproject.toml`、`src/`（含 `src/__main__.py` 的日志配置）与 LICENSE 文件。

| # | 引导要素 | 所在位置（README.md / 提交） | agent-web-search 是否已有 |
| --- | --- | --- | --- |
| 1 | 顶部 release + license 徽章 | L4-5（`9b08245`） | ❌ 无（顶部仅语言切换行） |
| 2 | 一段话定位 + 点名具体客户端（Claude Code、ZCode） | 开头（`ec34a87`） | ✅ 已有（README.md L5-8 点名 Claude Code、ZCode） |
| 3 | "为什么用它而不是付费/内置方案"显式章节（三条卖点 + 一段定位） | L14（`ec34a87`） | ⚠️ 卖点 bullets 已有（L10-20），但无"vs 付费搜索 API"的显式对比章节 |
| 4 | Quick start 三步编号：安装 → 注册 → 用起来 | L72-280（`ec34a87`） | ⚠️ 有三步（L80-192），但章节名为 `Install`，第 3 步是 `Verify` 而非 `Use it`（无状态检查、无冲突引导） |
| 5 | 分平台二进制下载表 + 推荐安装位置表（无需管理员权限、不必加 PATH） | L74 起（7 月一波已建，本次保留） | ✅ 已有（L88-110，且更详细） |
| 6 | `go install` 渠道折叠块（注明 Go 版本要求及二进制落点） | `<details>`（`ec34a87`） | N/A（Python 项目；且 PyPI 上 `agent-web-search` 0.1.0 为他人同名包——MIT、httpx+pydantic 客户端，名字已被占用，无 pip 直装渠道） |
| 7 | 源码构建折叠块（版本要求与 go.mod 对齐） | `<details>`（`ec34a87`） | ❌ 有"Build from source"但是独立 H2（L194），未折叠进安装步骤 |
| 8 | "你不需要自己启动 stdio server"新手误区说明（链接官方 transports） | 步骤 2（`ec34a87`） | ❌ 无显式说明（仅 L106-107 隐含"MCP 通过配置里的绝对路径启动二进制"） |
| 9 | 各客户端配置 `<details>` 折叠：ZCode / Claude Code / Zed / opencode（含 Windows 路径、CLI 一行命令） | L131-248（`ec34a87` 新增 Zed、opencode） | ⚠️ ZCode / Claude Code 有（L129-157）但平铺未折叠；无 Zed、opencode |
| 10 | Windows 专属提示（`.exe` 全路径、JSON 用正斜杠避免转义） | 步骤 2 引用块 | ✅ 已有（L173-179，含 SmartScreen 说明，比 fetch 更全） |
| 11 | 按客户端的"确认已注册"状态检查（Zed 绿点 / `opencode mcp list` / `claude mcp list`） | 步骤 3（`ec34a87`） | ❌ 无（L181 只说"重启后工具会出现"） |
| 12 | 可直接粘贴的首个示例提示词（点名 server 名） | 步骤 3（`ec34a87`） | ⚠️ 有示例提示词（L186-188）但只点名工具名 `web_search`，未点名 server |
| 13 | "agent 一直用内置工具怎么办"疑难引导（提示词点名 / AGENTS.md 一句话 / 关闭内置工具） | 步骤 3（`ec34a87`） | ❌ 完全没有；对本仓库最对症——Claude Code / ZCode 均有内置 WebSearch |
| 14 | Usage 章节：真实返回示例（含截断的真实输出）、参数示例、提示词→参数映射 | L281（`ec34a87`） | ⚠️ L27-38 有简略返回示例；无真实输出示例、无提示词→参数映射清单 |
| 15 | 与托管付费方案的对比表 + 时效脚注 | L311（`ec34a87`） | ❌ 无 |
| 16 | Troubleshooting：日志环境变量（`WEB_FETCH_LOG`，含按 server 注入 env 的 JSON 示例） | L325（原步骤 4 收编） | ⚠️ 无对应功能：本仓库日志固定 stderr / INFO（`src/__main__.py` L19），无环境变量开关；只能文档化现状或先加功能 |
| 17 | FAQ 折叠块（"免费有什么代价？""能在公司用吗？"） | L325 下（`ec34a87`） | ❌ 无 |
| 18 | Contributing：报告要素、本地开发命令、给 AI agent 的仓库导览（CONTEXT.md / docs/adr/ / AGENTS.md） | L372（`ec34a87`） | ❌ 无独立章节（仅 License 下有 Contribution Terms，L241-243） |
| 19 | 双语 README 逐节对齐（README.zh-CN.md 同构镜像） | 全部（`6d12efd` 建立，`ec34a87` 保持对齐） | ❌ 有实际缺口：README.zh-CN.md 缺整个 License 章节（EN 版 L234-243 有 License + Contribution Terms，中文版止于"工作原理"） |
| 20 | 文风统一：去 em dash、句式化标题、参数表默认值如实（"none" 而非 "—"） | 全篇（`ec34a87`） | ⚠️ 本仓库无 em-dash 风格约定，README 大量使用 em dash；Python 3.10+ 与 `pyproject.toml` 一致（无误）。另有内部矛盾：L86 "no archive, no extraction step" vs L192 "the binary you extracted"（zh 版 L135 同样残留"解压"措辞） |

## 7. 对照结论（agent-web-search 是否需要改）

**需要改，但不是照抄。** 分四类：

**A. 无争议 bug 修复（无论选哪档都该修）**

1. `README.zh-CN.md` 补齐缺失的整个 License / 贡献条款章节（与 EN 版 L234-243 对齐）。
2. README.md L192 与 README.zh-CN.md L135 的"你解压出来的二进制"改为与 L86 "无需解压" 一致的措辞。
3. `pyproject.toml` L6 `license = { text = "MIT" }` 与 LICENSE（AGPL-3.0）及 README 矛盾，属残留错误，应改为 AGPL-3.0-only 表述。

**B. 高价值新手引导缺口（fetch 这轮的核心增量，本仓库确实缺）**

第 13 项（内置 WebSearch 冲突引导，最对症）、第 8 项（无需自己启动 server）、第 11 项（按客户端注册状态检查）、第 1 项（徽章）、第 9 项（Zed / opencode 配置 + 折叠化）、第 7 项（源码构建折叠进安装步骤）、第 12 项（示例提示词点名 server）。

**C. 结构性重构（可选项，即完整 Pattern C 骨架）**

第 3 项（Why 章节）、第 4 项（Install → Quick start / Verify → Use it）、第 14 项（Usage 真实输出 + 映射）、第 15 项（对比表 + 时效脚注）、第 17 项（FAQ 折叠）、第 18 项（Contributing）。

**D. 不建议盲跟的项**

第 6 项（go install：Python 项目 N/A，且 PyPI 同名包已被他人占用）；第 16 项的 env 开关写法（本仓库无此功能，需先实现 `WEB_SEARCH_LOG` 之类开关才有文档可写，或仅文档化"日志在 stderr"）；第 20 项的 em-dash 清理（本仓库无此风格约定，属 fetch 仓库自身的风格决定）。

## 8. 落地记录（2026-10-31）

按用户决定（A + B + C 全量对齐，"参考不硬搬"），当日完成：

- `README.md` / `README.zh-CN.md` 按 Pattern C 骨架重写：Why（vs 付费搜索 API）/ Quick start 三步（Install / Register / Use it，含"无需自己启动 stdio server"说明、ZCode / Claude Code / Zed / opencode 四个 `<details>` 配置块）/ Usage（真实输出，来自当日对 `orchestrate()` 的实跑）/ Comparison（对比托管搜索 API，定性 + 时效脚注）/ Troubleshooting（含 stderr 日志说明与两条 FAQ 折叠）/ Contributing。
- 内置搜索工具冲突引导按本仓库事实写入：Claude Code `permissions.deny: ["WebSearch"]`、Zed agent profile（内置 `search_web` 为 Pro 专属，免费计划正好用本工具补位）、opencode `"tools": { "websearch": false }`（均经当日一手核实，来源：code.claude.com permissions 文档、zed.dev tools/agent-profiles 文档、opencode.ai tools 文档）。
- A 类 bug 一并修复：中文版补齐 License / 贡献条款章节；"解压出来的二进制"措辞消除；`pyproject.toml` license 由 MIT 残留改为 `AGPL-3.0-only`。
- 全文按 unslop 规范过了一遍（用户要求）：正文 em dash / 中文"——"清零（参数表占位符一并改为 none / 无），加粗引导从"标签："改为"标签。"句式，删标语式句子（"第一次就能连上，每次都能"），拆长句。第 20 项"去 em dash"由此从"不建议盲跟"转为跟进。
- 用户复审后追加两条硬性规范并已落实：① 不得用括号做插入语替代破折号（unslop 规则 13 原文即 "periods or commas only, no parentheses"），全部插入语拆成独立句子，仅保留 markdown 链接、代码块、函数签名与 "Copyright (c)" 中的括号；② README 正文删除全部 ADR-XXXX 决策号引用，`CONTEXT.md` / `docs/adr/` 导览只保留在 Contributing 一节。
- 验证：`python -m pytest -m "not integration"` 24 passed；toml 解析正常；双语 `<details>` 标签 7 开 7 闭配对；全文扫描无残留 em dash / 弯引号。

## 6. 可供 agent-web-search 参考的做法（观察，非结论）

1. **先研究后改稿**：`ec34a87` 之前先落地 `docs/research/readme-onboarding-patterns.md`（8 个高星仓库的一手 README 调研 + 明确的骨架建议），再按建议重构。agent-web-search 若要跟进，可直接复用该研究结论，也可沿用"研究笔记先行"的做法。
2. **结构骨架**（Pattern C，browser-tools-mcp 式）：定位（Why this instead of X）→ 三步 Quick start（Install / Register / Use it）→ Usage（真实输出示例）→ Comparison（带时效脚注）→ What it does not do → Troubleshooting（含 FAQ 折叠）→ How it works → Contributing → License。新手在首屏到一屏半内即可完成"装好并跑通第一次调用"。
3. **新手误区专项**：两处高价值澄清——"stdio server 不需要自己启动"、"注册后模型仍可能选内置工具（含关闭/点名办法）"。agent-web-search 同为替代客户端内置（搜索）工具的 stdio MCP server，第二点尤其对症。
4. **配置折叠化**：多客户端配置用 `<details>` 收纳，主流程只留一个 canonical JSON；新增了 Zed、opencode 两个客户端覆盖面。
5. **术语与决策先行**：新能力（passthrough、size cap）先写 ADR + CONTEXT.md 术语，再在 README/工具 schema 同步呈现，三者措辞一致（`553fc85` 专门校准真实性）。
