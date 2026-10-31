# agent-web-search

[![GitHub release](https://img.shields.io/github/v/tag/ChHsiching/agent-web-search?include_prereleases&sort=semver)](https://github.com/ChHsiching/agent-web-search/releases/latest)
[![License](https://img.shields.io/github/license/ChHsiching/agent-web-search)](https://github.com/ChHsiching/agent-web-search/blob/main/LICENSE)

[English](./README.md) | **简体中文**

一个面向 MCP-capable agent 的**免费、不限量的网页搜索**，适用于 Claude Code、ZCode、Zed、opencode，以及任何支持 Model Context Protocol 的客户端。它执行搜索查询，返回带页面正文摘录的结果，agent 可直接阅读。无需 API key，没有配额，没有月度限额。

## 为什么用它，而不是付费搜索 API

- **免费且不限量。** 通过普通 HTTP 搜索 DuckDuckGo。无需 API key，没有配额，没有月度限额，不会遇到 429。
- **通用 MCP 工具。** 暴露一个名为 `web_search` 的工具，参数采用 agent 已熟悉的标准 schema，即 query、domain filter、recency、content size、location。在任何 MCP 客户端里即插即用，无需改提示词。
- **启动可靠。** 以单个自包含二进制分发，Python 解释器封装在内。不用 `npx`，无需安装运行时，启动时不联网。
- **近乎零维护。** 对 DuckDuckGo 的访问通过 `ddgs` 库进行，由它处理反爬、限流、重试。没有要跟进的抓取引擎。

托管式搜索 API 把每一次查询都送去厂商云端，让你保管一个 API key，再按请求计量收费，想提额就得加钱。本工具在你的机器上完成同样的工作，每次查询 $0。它走普通 HTTP 访问 DuckDuckGo，跑在你的 MCP 客户端本就管理的进程里。

## 它做什么

给它一个搜索查询，返回带页面正文摘录的排序结果列表：

```
web_search({ "search_query": "rust tokio tutorial" })
  → [{ "title": "Tutorial | Tokio", "url": "https://tokio.rs/tokio/tutorial",
       "summary": "Tokio is an asynchronous runtime for the Rust …",
       "site_name": "tokio.rs", "favicon": "https://tokio.rs/favicon.ico" },
     …]
```

排名靠前的结果会被抓取页面正文并通过 Readability 引擎提取，这样 agent 能读到实际内容，而不仅仅是一小段 snippet。靠后的结果保留来源自身的 snippet。我们返回的是内容，不是摘要。没有模型调用，没有摘要加工。

### 工具参数

| 参数                     | 必填 | 默认值    | 说明                                                              |
| ------------------------ | ---- | --------- | ----------------------------------------------------------------- |
| `search_query`           | 是   | 无        | 搜索关键词。                                                      |
| `search_domain_filter`   | 否   | 无        | 限定到某个域名，如 `docs.rust-lang.org`。                         |
| `search_recency_filter`  | 否   | `noLimit` | `oneDay`、`oneWeek`、`oneMonth`、`oneYear`、`noLimit`。           |
| `content_size`           | 否   | `medium`  | `medium` 约摘录 500 词，`high` 约摘录 2500 词。                   |
| `location`               | 否   | `cn`      | `cn` 或 `us`。                                                    |

### 它**不**做什么

- **不做网页抓取 / reader。** 它做搜索，不抓取任意 URL。抓取页面是另一个工具的事，见 [`agent-web-fetch`](https://github.com/ChHsiching/agent-web-fetch)。
- **不做摘要 / 翻译。** 它返回页面正文，不加工内容。没有付费模型调用。
- **不做图片 / 新闻 / 视频搜索。** 只做通用网页结果，覆盖 agent 绝大多数需求。
- **不暴露 `max_results`。** 结果数量固定为 10 条左右且不对外暴露，与目标工具一致。

### 已知限制

这些都源于免费、轻依赖的设计取舍，不是 bug：

- **recency 过滤是尽力而为。** `search_recency_filter` 会被透传给上游 Source，由它宽松执行。返回结果大体偏新但不会被严格限定在窗口内。这是上游行为，除非换上更重的 Source 否则无法收紧。
- **没有发布时间字段。** 每条结果只含 title、url、summary、site_name、favicon。上游 Source 不返回时间戳，所以我们无从暴露。这是底层数据的范围，不是漏掉的功能。
- **质量取决于查询形态。** 关键词查询如 `rust tokio spawn example` 的相关性，远高于完整自然语言问句，中文查询尤其明显。工具描述已引导模型传关键词；在提示词里鼓励关键词查询会进一步改善效果。
- **不做事件聚类 / 结构化摘要。** 超出范围。本工具返回扁平的 ranked list + 页面正文摘录。

## 快速开始

### 1. 安装对应平台的二进制

从[最新 release](../../releases) 下载适合你平台的文件。它是一个单一自包含二进制，Python 解释器和所有依赖都封装在内，所以没有压缩包，也无需解压：

| 平台 | 文件 |
| --- | --- |
| Windows | `agent-web-search-windows-x64.exe` |
| Linux | `agent-web-search-linux-x64` |
| macOS | `agent-web-search-macos` |

无需安装器，无需安装运行时。不需要 Node，也不需要 Python。

**文件放哪里。** 每个平台都有一个推荐的用户级位置，不需要管理员权限，是用户自己安装的程序约定俗成的存放点：

| 平台 | 推荐位置 |
| --- | --- |
| Windows | `%LOCALAPPDATA%\Programs\agent-web-search\agent-web-search.exe` |
| macOS / Linux | `~/.local/bin/agent-web-search` |

不过 MCP 其实不在乎文件放在哪。它通过你配置里的绝对路径来启动二进制，所以放在任何你有读/执行权限的地方都行，也**不需要**加到 `PATH`。只要别放进其他用户的目录，或需要管理员权限的系统文件夹。下面的示例用的是推荐位置；如果你放在别处，把路径替换掉即可。

<details>
<summary>改用从源码构建，需要 Python 3.10+</summary>

```sh
# 以 editable 模式安装（含开发依赖）
pip install -e .

# 通过 PyInstaller 生成独立二进制
pip install pyinstaller
pyinstaller agent-web-search.spec --noconfirm
# → dist/agent-web-search（Windows 下是 .exe）

# 跑测试套件
python -m pytest
```

每个 release 二进制都是 PyInstaller 打包，Python 解释器和所有依赖封装在一个文件里。

</details>

### 2. 在你的 MCP 客户端注册它

这是一个标准的 **stdio MCP server**，没有参数，没有环境要求。**你永远不需要自己启动它。** 手动运行二进制只会阻塞等待 stdin。你的 MCP 客户端会按需启停这个 server。[MCP transports 文档](https://modelcontextprotocol.io/docs/concepts/transports) 解释了 stdio server 的工作方式。你要做的只是把客户端配置指向这个二进制。

每个 MCP 客户端里的配置项思路都一样。把 `command` 指向二进制的绝对路径，`args` 留空：

```json
"chhsich-web-search": {
  "type": "stdio",
  "command": "/绝对路径/agent-web-search",
  "args": []
}
```

各客户端之间唯一的区别是这条记录**放在哪**以及确切的 key 名。下面是常见客户端的具体示例：

<details>
<summary><strong>ZCode：</strong>按 server 名为 key 的扁平 object</summary>

把记录加到它的 MCP servers 配置，即按 server 名为 key 的扁平 object，没有外层包装：

```json
{
  "chhsich-web-search": {
    "type": "stdio",
    "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
    "args": []
  }
}
```
</details>

<details>
<summary><strong>Claude Code：</strong><code>~/.claude.json</code> 的 <code>mcpServers</code> key 下</summary>

server 放在 `~/.claude.json` 的 `mcpServers` key 下，Windows 下这个文件是 `%USERPROFILE%\.claude.json`：

```json
{
  "mcpServers": {
    "chhsich-web-search": {
      "type": "stdio",
      "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
      "args": []
    }
  }
}
```

或从 CLI 添加，效果相同：`claude mcp add chhsich-web-search "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe"`
</details>

<details>
<summary><strong>Zed：</strong><code>context_servers</code> key，没有 <code>type</code> 字段</summary>

最简单走 UI：**Settings → AI → MCP Servers → Add Server → Add Local Server**，指向 exe。或者编辑设置文件：命令面板运行 `zed: open settings file`，Windows 下位于 `C:\Users\<username>\AppData\Roaming\Zed\settings.json`。

```json
{
  "context_servers": {
    "chhsich-web-search": {
      "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
      "args": []
    }
  }
}
```

注意：Zed 内置的 `search_web` 工具需要 Zed Pro 订阅。免费计划下可以用本工具获得同等能力。
</details>

<details>
<summary><strong>opencode：</strong><code>mcp</code> key、<code>"type": "local"</code>、<code>command</code> 是数组</summary>

配置在项目根目录的 `opencode.json`，或全局的 `~/.config/opencode/opencode.json`。Windows 下是 `%USERPROFILE%\.config\opencode\opencode.json`。

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "chhsich-web-search": {
      "type": "local",
      "command": ["C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe"]
    }
  }
}
```
</details>

**任何其他 stdio MCP 客户端。** 找到它存放 MCP server 列表的地方，无论是 JSON/YAML 配置还是设置界面，加一条记录：type `stdio`，`command` = 二进制的绝对路径，`args` = `[]`。这就是全部契约，没有别的参数要设。

> **替换路径。** 上面的示例用的是推荐安装位置，`<username>` 是占位符。替换成你的实际用户名，或如果你的客户端支持环境变量展开就用 `%LOCALAPPDATA%`。如果你把二进制放在别处，相应调整路径。

> **命名。** key，即示例里的 `chhsich-web-search`，是你在客户端这边给 server 的标签，叫什么都行。它暴露的工具永远叫 `web_search`，所以在任何客户端里模型调用的都是 `web_search(...)`。

> **Windows 路径提示。** 用包含 `.exe` 的完整绝对路径。JSON 里用正斜杠即可，能避免反斜杠转义。

> **Windows SmartScreen 提示。** release 二进制未签名，所以 Windows 首次运行时可能弹出"Windows 已保护你的电脑"提示。点 **更多信息 → 仍要运行**。未签名二进制都会这样，且只发生一次。

编辑配置后重启客户端。`web_search` 工具就会和内置工具一起出现，模型可以像调用其他工具一样调用它。

### 3. 用起来

先确认 server 确实注册上了。每个客户端都有状态视图：

- **Zed**：打开 Settings → AI → MCP Servers，`chhsich-web-search` 旁边的圆点应为绿色，显示 "Server is active"。
- **opencode**：在终端运行 `opencode mcp list`。
- **Claude Code**：运行 `claude mcp list`。

然后让模型搜索任意内容，**点名 server**，让它优先选这个工具而不是任何内置工具：

> 用 chhsich-web-search 的 web_search 工具搜索 "rust async runtime"

每条结果包含 title、URL、site name、favicon 和 summary。靠前结果的 summary 是页面正文摘录。

**如果 agent 一直用它自带的网页搜索。** Claude Code、Zed、opencode 都自带网页搜索工具，注册本 server **不会**自动禁用它们。模型可能继续选内置工具，表现和"工具没生效"一模一样，而注册其实是好的。两种解决办法：

- 在提示词里点名 server，如上，或在 `AGENTS.md` / 规则文件里加一句："需要搜索网页时，用 `chhsich-web-search` 的 `web_search` 工具。"
- 或者直接关掉内置工具：
  - **Claude Code**：在 `~/.claude/settings.json` 的 `permissions.deny` 里加 `"WebSearch"`。裸工具名会把它从模型上下文中整个移除。
  - **Zed**：创建一个 [agent profile](https://zed.dev/docs/ai/agent-profiles)，关掉内置 `search_web`，只开本 server 的工具。免费计划下 `search_web` 本就不可用。
  - **opencode**：在 `opencode.json` 用 `tools` 键禁用，写成 `"tools": { "websearch": false }`。

如果 server 一直不出现或调用失败，见[故障排查](#故障排查)。

## 用法

这是对 `rust tokio tutorial` 的一次真实查询输出，约 10 条结果中的第一条。此处 `summary` 是页面正文摘录，为展示做了截断：

```json
{
  "title": "Tutorial | Tokio - An asynchronous Rust runtime",
  "url": "https://tokio.rs/tokio/tutorial",
  "summary": "Tokio is an asynchronous runtime for the Rust programming
    language. It provides the building blocks needed for writing networking
    applications. It gives the flexibility to target a wide range of systems,
    from large servers with dozens of cores to small embedde …",
  "site_name": "tokio.rs",
  "favicon": "https://tokio.rs/favicon.ico"
}
```

提示词里的说法对应到参数：

| 你说 …… | 参数 |
| --- | --- |
| "只要 `docs.rust-lang.org` 的结果" | `search_domain_filter: "docs.rust-lang.org"` |
| "只要最近一周的" | `search_recency_filter: "oneWeek"` |
| "多给我一些正文" | `content_size: "high"` |
| "搜英文资料" | `location: "us"` |

关键词查询优于问句。`rust tokio spawn example` 的相关性远高于"如何在 rust 里用 tokio 生成一个任务？"，中文查询尤其明显。见[已知限制](#已知限制)。

## 对比

|  | agent-web-search | 托管式搜索 API，如 Tavily、Serper、Brave Search API |
| --- | --- | --- |
| API key | 不需要 | 必需 |
| 费用 | 每次查询 $0，不限量 | 按量计费，配额随套餐 |
| 查询与页面内容的流向 | 你的机器 → 直连 DuckDuckGo 和目标站点 | 经过厂商云端 |
| 运行时 | 单个自包含二进制 | 账号、key 管理、SDK 更新 |
| 发布时间、图片、新闻等附加结果 | 无，扁平的网页排序结果 | 因服务和套餐而异 |
| 许可证 | AGPL-3.0，双许可 | 商业 SaaS 条款 |

以上为 2026 年 10 月时点的状况。请以各服务当前定价为准。

## 故障排查

- **工具一直不出现。** 确认客户端配置的 `command` 是你下载的二进制的绝对路径，Windows 下含 `.exe`。然后重启客户端，再用上面第 3 步的状态视图检查。
- **调用立刻失败。** 无论限流、空结果还是页面抓取错误，每一种失败都会作为结构化响应返回给模型，进程永不崩溃。上游瞬时限流通常稍候自行恢复，等一会儿重试即可。
- **结果空或离题。** 用关键词查询代替自然语言问句，知道来源时加域名过滤。见[已知限制](#已知限制)。
- **日志。** server 以 INFO 级别把日志写到 stderr，不写任何文件。stdout 只走 JSON-RPC。能显示 MCP server stderr 的客户端会在那里展示日志。

<details>
<summary><strong>FAQ：免费？有什么代价？</strong></summary>

没有暗坑，只有取舍。搜索通过 `ddgs` 库走普通 HTTP 访问 DuckDuckGo，没有 key，没有账单。代价是接受上面的限制：recency 过滤尽力而为、没有发布时间字段、重度使用会撞上游限流。

</details>

<details>
<summary><strong>FAQ：能在公司用吗？</strong></summary>

能。AGPL-3.0 下内部商业使用免费。分发本工具或将其作为网络服务提供时，衍生作品须以 AGPL-3.0 开源；闭源商业使用需另行获得商业许可。见[许可证](#许可证)。

</details>

## 工作原理

```
query → 参数映射 → DuckDuckGo（ddgs：反爬/重试已处理）
     → 结果列表（每条含 title/url/snippet）
     → 靠前结果：抓取页面 → Readability 提取 → 按字数截断的正文
                                                  ↘ snippet 兜底（永不返回空）
     → 组装 {title, url, summary, site_name, favicon} → JSON
```

搜索和页面抓取都走依赖注入的接缝，所以 fan-out、提取、组装这些核心逻辑无需联网就能单测。每一种失败都作为结构化响应返回，模型能读懂，server 进程永不崩溃。

早期版本曾基于 SearXNG 公共实例实现，用的是 Rust，但实测发现 38 个健康实例无一可用，于是转用 DuckDuckGo 加 `ddgs` 库。

## 参与贡献

欢迎 issue 和 PR。issue 以 GitHub issues 的形式管理，约定见 `docs/agents/issue-tracker.md`。一个好的缺陷报告应包含：确切的查询、传入的参数、期望结果、实际返回。

本地开发：

```sh
pip install -e .
python -m pytest          # 带 integration 标记的测试会访问真实网络
```

如果你是 AI agent 或新贡献者，可以从 `CONTEXT.md` 的术语表、`docs/adr/` 的架构决策和 `AGENTS.md` 的仓库约定入手。

提交 PR 即表示你同意[贡献条款](#贡献条款)。

## 许可证

AGPL-3.0-only · Copyright (c) 2026 ChHsiching，见 [LICENSE](LICENSE)。

- 使用，含内部商业使用、修改和分发均免费。分发本工具或将其作为网络服务提供时，衍生作品须以 AGPL-3.0 开源。
- 闭源商业使用需另行获得商业许可：hsichingchang@gmail.com

### 贡献条款

提交 PR 即表示你同意以 AGPL-3.0 许可你的贡献，并授予维护者提供独立商业许可的权利。你的贡献仍以 AGPL 向所有人开放。
