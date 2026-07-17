# agent-web-search

[English](./README.md) | **简体中文**

一个面向 MCP-capable agent（Claude Code、ZCode，以及任何支持 Model Context Protocol 的客户端）的**免费、不限量的网页搜索**。它执行搜索查询，返回带页面正文摘录的结果，agent 可直接阅读 —— 是众多 agent 客户端内置的付费、限流 `web_search_prime` 工具的替代品。

- **免费且不限量** —— 通过普通 HTTP 搜索 DuckDuckGo。无需 API key，没有配额，没有月度限额，不会遇到 429。
- **Drop-in 兼容** —— 暴露一个名为 `web_search` 的工具，参数和付费 `web_search_prime` 一致，所以 agent 的提示词无需改动（工具名不同，但 agent 按描述选工具，不按硬编码名字）。
- **启动可靠** —— 以单个自包含二进制分发（Python 解释器封装在内）。不用 `npx`，无需安装运行时，启动时不联网。第一次就能连上，每次都能。
- **近乎零维护** —— 对 DuckDuckGo 的访问通过 `ddgs` 库进行，它替我们处理反爬、限流、重试。没有要跟进的抓取引擎。

## 它做什么

给它一个搜索查询，返回带页面正文摘录的排序结果列表：

```
web_search({ "search_query": "rust tokio tutorial" })
  → [{ "title": "Tutorial | Tokio", "url": "https://tokio.rs/tokio/tutorial",
       "summary": "Tokio is an asynchronous runtime for the Rust …",
       "site_name": "tokio.rs", "favicon": "https://tokio.rs/favicon.ico" },
     …]
```

排名靠前的结果会被抓取页面正文并提取（通过 Readability 引擎），这样 agent 能读到实际内容，而不仅仅是一小段 snippet。靠后的结果保留来源自身的 snippet。我们返回的是内容，不是摘要 —— 没有模型调用，没有摘要加工。

### 工具参数

| 参数                     | 必填 | 默认值    | 说明                                                              |
| ------------------------ | ---- | --------- | ----------------------------------------------------------------- |
| `search_query`           | 是   | —         | 搜索关键词。                                                      |
| `search_domain_filter`   | 否   | —         | 限定到某个域名，如 `docs.rust-lang.org`。                         |
| `search_recency_filter`  | 否   | `noLimit` | `oneDay`、`oneWeek`、`oneMonth`、`oneYear`、`noLimit`。           |
| `content_size`           | 否   | `medium`  | `medium`（约 500 词摘录）或 `high`（约 2500 词摘录）。            |
| `location`               | 否   | `cn`      | `cn` 或 `us`。                                                    |

### 它**不**做什么

- **不做网页抓取 / reader** —— 它做搜索；不抓取任意 URL。（那是另一个工具 —— 见 [`agent-web-fetch`](https://github.com/ChHsiching/agent-web-fetch)。）
- **不做摘要 / 翻译** —— 它返回页面正文，不加工内容。没有付费模型调用。
- **不做图片 / 新闻 / 视频搜索** —— 只做通用网页结果，覆盖 agent 绝大多数需求。
- **不暴露 `max_results`** —— 结果数量固定（约 10 条）且不对外暴露，与目标工具一致。

### 已知限制

这些都源于免费、轻依赖的设计取舍（ADR-0006），不是 bug：

- **recency 过滤是尽力而为** —— `search_recency_filter` 会被透传给上游 Source，由它宽松执行。返回结果大体偏新但不会被严格限定在窗口内；这是上游行为，除非换上更重的 Source 否则无法收紧。
- **没有发布时间字段** —— 每条结果只含 title / url / summary / site_name / favicon。上游 Source 不返回时间戳，所以我们无从暴露（这是底层数据的范围，不是漏掉的功能）。
- **质量取决于查询形态** —— 关键词查询（如 `rust tokio spawn example`）的相关性远高于完整自然语言问句，中文查询尤其明显。工具描述已引导模型传关键词（ADR-0007）；在提示词里鼓励关键词查询会进一步改善效果。
- **不做事件聚类 / 结构化摘要** —— 超出范围。本工具返回扁平的 ranked list + 页面正文摘录。

## 安装

### 1. 下载对应平台的二进制

从[最新 release](../../releases) 下载适合你平台的文件 —— 是一个单一自包含二进制（Python 解释器和所有依赖都封装在内，所以没有压缩包，无需解压）：

| 平台 | 文件 |
| --- | --- |
| Windows | `agent-web-search-windows-x64.exe` |
| Linux | `agent-web-search-linux-x64` |
| macOS | `agent-web-search-macos` |

无需安装器，无需安装运行时（不需要 Node 或 Python）。

**文件放哪里：** 每个平台都有一个**推荐的用户级位置** —— 不需要管理员权限，是用户自己安装的程序约定俗成的存放点：

| 平台 | 推荐位置 |
| --- | --- |
| Windows | `%LOCALAPPDATA%\Programs\agent-web-search\agent-web-search.exe` |
| macOS / Linux | `~/.local/bin/agent-web-search` |

不过 MCP 其实不在乎文件放在哪 —— 它是通过你配置里的绝对路径来启动二进制的，所以放在任何你有读/执行权限的地方都行（也**不需要**加到 `PATH`）。只要别放进其他用户的目录，或需要管理员权限的系统文件夹。下面的示例用的是推荐位置；如果你放在别处，把路径替换掉即可。

### 2. 在你的 MCP 客户端注册它

这是一个标准的 **stdio MCP server**：没有参数，没有环境要求。每个 MCP 客户端里的配置项思路都一样 —— 把 `command` 指向二进制的绝对路径，`args` 留空：

```json
"chhsich-web-search": {
  "type": "stdio",
  "command": "/绝对路径/agent-web-search",
  "args": []
}
```

各客户端之间唯一的区别是这条记录**放在哪**以及确切的 key 名。下面是常见客户端的具体示例：

**ZCode** —— 把记录加到它的 MCP servers 配置（一个按 server 名为 key 的扁平 object，没有外层包装）：

```json
{
  "chhsich-web-search": {
    "type": "stdio",
    "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
    "args": []
  }
}
```

**Claude Code** —— `~/.claude.json`（Windows 下是 `%USERPROFILE%\.claude.json`），server 放在 `mcpServers` key 下：

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

或通过 CLI（效果相同）：`claude mcp add chhsich-web-search "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe"`

**任何其他 stdio MCP 客户端** —— 找到它存放 MCP server 列表的地方（JSON/YAML 配置、设置界面等），加一条记录：type `stdio`，`command` = 二进制的绝对路径，`args` = `[]`。这就是全部契约 —— 没有别的参数要设。

> **替换路径：** 上面的示例用的是推荐安装位置，`<username>` 是占位符 —— 替换成你的实际用户名（或如果你的客户端支持环境变量展开，用 `%LOCALAPPDATA%`）。如果你把二进制放在别处，相应调整路径。

> **命名：** key（上面的 `chhsich-web-search`）是你在客户端这边给 server 的标签 —— 叫什么都行；它不会和官方 `web-search-prime` 那条冲突，两者可以并存。它暴露的工具名为 `web_search`（刻意与付费的 `web_search_prime` 区分 —— 我们描述功能，不模仿名字）。

> **路径提示（Windows）：** 用包含 `.exe` 的完整绝对路径。JSON 里用正斜杠即可，能避免反斜杠转义。

> **Windows SmartScreen 提示：** release 二进制未签名，所以 Windows 首次运行时可能弹出"Windows 已保护你的电脑"提示。点 **更多信息 → 仍要运行**。未签名二进制都会这样，且只发生一次。

编辑配置后重启客户端。`web_search` 工具就会和内置工具一起出现，模型可以像调用其他工具一样调用它。

### 3. 验证它工作

重启客户端后，让模型搜索任意内容，例如：

> 用 web_search 工具搜索 "rust async runtime"

你应该得到一组结果，每条包含 title、url、summary、site name、favicon。如果工具缺失或返回空，检查 `command` 路径是否指向你解压出来的二进制。

## 从源码构建

需要 Python 3.10+。

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

每个 release 二进制都是 PyInstaller 打包 —— Python 解释器和所有依赖封装在一个文件里。

## 工作原理

```
query → 参数映射 → DuckDuckGo（ddgs：反爬/重试已处理）
     → 结果列表（每条含 title/url/snippet）
     → 靠前结果：抓取页面 → Readability 提取 → 按字数截断的正文
                                                  ↘ snippet 兜底（永不返回空）
     → 组装 {title, url, summary, site_name, favicon} → JSON
```

搜索和页面抓取都走依赖注入的接缝，所以核心逻辑（fan-out、提取、组装）无需联网就能单测。每一种失败（限流、空结果、页面抓取错误）都作为结构化响应返回，模型能读懂 —— server 进程永不崩溃。

项目术语表见 `CONTEXT.md`，架构决策见 `docs/adr/`。说明：早期版本曾基于 SearXNG 公共实例（用 Rust 实现），但实测发现 38 个健康实例无一可用 —— 项目转而使用 DuckDuckGo + `ddgs`（见 ADR-0006）。
