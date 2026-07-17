> 其他语言：[English](README.md) · **简体中文**

# agent-web-search

一个**免费、不限量、稳定**的 web 搜索 MCP 工具，供编程 agent（Claude Code、Codex、ZCode）使用。它是付费/托管的 `web_search_prime` 的 drop-in（即插即用）替代品——同名、同参数、零成本。

它通过 **DuckDuckGo** 搜索（使用 [`ddgs`](https://github.com/deedy5/duckduckgo_search) 库，自动处理反爬/限流），无需 API key、无需 Docker、无按次计费。结果附带页面正文摘录，让 agent 能直接阅读每条结果。

## 为什么做这个

官方的 `web_search_prime` 工具按量计费，session 中途经常返回 `429 Weekly/Monthly Limit Exhausted`。这个项目存在的意义是：让开发者给自己的 agent 一个**永远不会用完**的搜索能力。

## 安装

**方式 A — 预编译二进制（推荐，无需 Python）：**

1. 从 [Releases](../../releases) 下载对应平台的压缩包：
   - Windows：`agent-web-search-windows-x64.zip`
   - Linux：`agent-web-search-linux-x64.tar.gz`
   - macOS：`agent-web-search-macos.tar.gz`
2. 解压，得到单个二进制文件 `agent-web-search`（Windows 下是 `.exe`）。
3. 记下它的完整路径，例如 `C:\Tools\agent-web-search.exe` 或 `/home/you/bin/agent-web-search`。
4. 用该路径配置你的 agent（见下）。

这个二进制是 PyInstaller 打包的——Python 解释器和所有依赖都封装在里面，所以你**不需要**安装 Python。（你也可以把二进制放进 `PATH`，然后用 `agent-web-search` 这个名字作为 command。）

**方式 B — 从源码安装（需要 Python 3.10+）：**

```sh
git clone https://github.com/ChHsiching/agent-web-search.git
cd agent-web-search
pip install -e .
```

这会在你的 `PATH` 上安装一个 `agent-web-search` 脚本。下面的配置里 command 直接用这个名字即可。

## 配置你的 agent

这是一个标准的 stdio MCP server。把它加到 agent 的 MCP 配置里——下面的结构对 **ZCode、Claude Code、Codex** 都通用（都读取同样的 `mcpServers` 块）：

```json
{
  "mcpServers": {
    "chhsich-web-search": {
      "type": "stdio",
      "command": "/完整路径/agent-web-search",
      "args": []
    }
  }
}
```

- **`command`**：你解压出来的二进制的完整路径（方式 A），或者它在 `PATH` 上时直接用 `agent-web-search`（方式 B）。
- **`args`**：留空——server 不接受任何参数。
- **`type`**：必须是 `"stdio"`。

server key（这里是 `chhsich-web-search`）是你自己的标签，叫什么都行。默认的 `chhsich-web-search` 带了作者命名空间，不会和官方 `web-search-prime` 或别人的 key 冲突。

> ⚠️ **如果你还配置着官方那个 `web-search-prime`，不要复用这个 key**——两个同名 key 会冲突，其中一个会被无声覆盖。想完全*替代*官方工具，请先把官方那条删掉或改名，再用 `web-search-prime`。

这个 server 暴露的 tool 名是 **`web_search_prime`**——和付费版完全同名、同参数。所以无论你选哪个 server key，agent 的提示词和工具调用都无需改动。

> 注意：如果同时加载的两个 server 都暴露名为 `web_search_prime` 的 tool，agent 的行为是未定义的（通常一个会遮蔽另一个）。请只保留其中一个配置。

**各 agent 的配置文件位置**（把上面的块放进去）：

- **ZCode**：你的 ZCode MCP 配置（具体文件见 ZCode 文档）。
- **Claude Code**：`~/.claude.json`（或项目级 `.mcp.json`）。
- **Codex**：你的 Codex MCP servers 配置。

配置完成后重启 agent。它会发现 `web_search_prime` 工具。

## 工作原理

- **搜索后端：** DuckDuckGo，通过 `ddgs` 库访问——这是实测中唯一稳定可靠的免费搜索后端。`ddgs` 处理反爬、限流、重试，我们不用自己操心。
- **免费、无 key：** DuckDuckGo 搜索免费，`ddgs` 开源。无需注册，无需付费。
- **稳定性优先：** MCP `initialize` 握手前不做任何网络请求（打包二进制启动约 1 秒），stdout 只承载 JSON-RPC，错误优雅降级。
- **结果：** 每条结果包含 `title`、`url`、`summary`（前 3 条是页面正文摘录，其余是来源 snippet）、`site_name`、`favicon`。agent 直接读原始文本——我们不做任何摘要。

## 工具参数

| 参数 | 必填 | 说明 |
| --- | --- | --- |
| `search_query` | 是 | 搜索关键词。 |
| `search_domain_filter` | 否 | 限定到某个域名，如 `docs.rust-lang.org`。 |
| `search_recency_filter` | 否 | `oneDay`、`oneWeek`、`oneMonth`、`oneYear`、`noLimit`（默认）。 |
| `content_size` | 否 | `medium`（默认，约 500 词摘录）或 `high`（约 2500 词）。 |
| `location` | 否 | `cn`（默认）或 `us`。 |

## 从源码构建（PyInstaller 打包）

如果你想自己生成独立二进制：

```sh
pip install pyinstaller
pip install -e .
pyinstaller agent-web-search.spec --noconfirm
```

生成的二进制在 `dist/agent-web-search`（Windows 下是 `.exe`）。

## 设计决策

架构决策记录在 [`docs/adr/`](docs/adr/)，领域术语表在 [`CONTEXT.md`](CONTEXT.md)。完整规格见 issue #1。特别说明：早期版本曾基于 SearXNG 公共实例并用 Rust 实现，但实测发现 38 个健康实例无一可用，于是切换到 DuckDuckGo + `ddgs`（见 ADR-0006）。
