# 调研笔记：shields.io 徽章写法（README 顶部 release / license 徽章）

- 调研日期：2026-10-31
- 调研对象：本仓库 `README.md` 第 3-4 行与 `README.zh-CN.md` 第 4-5 行的两枚徽章（任务描述写"第 5-6 行"，实际行号以文件为准）
- 调研方式：只读。结论全部溯到一手来源：badges/shields 官方仓库源码（master 分支原文）、shields.io 官方文档页、docs.github.com、licensee 官方仓库，外加对 img.shields.io 与 api.github.com 的 curl / gh api 实测。未引用任何博客转述。未改动任何仓库文件（本报告除外）。
- 用户抱怨：① 版本徽章左标签显示 "tag"，期望 "version"；② 两枚徽章都是橙色，观感奇怪。

## 1. `github/v/tag` 与 `github/v/release` 的区别

| | `github/v/tag` | `github/v/release` |
| --- | --- | --- |
| 数据源 | GitHub GraphQL `refs(refPrefix: "refs/tags/")`，按 tag 的提交日期倒序。读的是 **git tag**，与是否创建 GitHub Release 无关 | 默认走 REST `GET /repos/{user}/{repo}/releases/latest`（即 GitHub 官方"latest release"语义，排除 prerelease 与 draft）；带 `include_prereleases` / `sort` / `filter` 任一参数时改拉 `/releases?per_page=100` 再选 |
| 默认左标签 | `tag`（源码 `static defaultBadgeData = { label: 'tag' }`） | `release`（源码 `static defaultBadgeData = { label: 'release' }`） |
| 无对应数据时 | 抛 `NotFound`：`no tags found` | 抛 `NotFound`：`no releases or repo not found`（徽章右侧显示该红字） |
| 专属查询参数 | `sort=semver`（用 `latest()` 语义排序）；`include_prereleases` 仅在 `sort=semver` 下有意义（控制预发布是否参选） | `display_name=tag|release`（默认 `tag`，显示 tag 名；`release` 显示 Release 标题）；`sort` / `include_prereleases` / `filter` 与 v/tag 共用同一 schema |
| 显示内容 | tag 名（经 `addv()` 自动补 `v` 前缀） | 默认 tag 名；`display_name=release` 时显示 Release 标题（无标题则回落 tag 名） |

来源：

- https://github.com/badges/shields/blob/master/services/github/github-tag.service.js
- https://github.com/badges/shields/blob/master/services/github/github-release.service.js
- https://github.com/badges/shields/blob/master/services/github/github-common-release.js（注释原话："Fetch the 'latest' release as defined by the GitHub API"，随后请求 `/releases/latest`）
- 文档页（构建器）：https://shields.io/badges/git-hub-tag 、https://shields.io/badges/git-hub-release （均 HTTP 200）

**结论 1**：两个端点的默认左标签分别是 "tag" 和 "release"，**都不会**自动显示 "version"；要显示 "version" 必须加 `?label=version`（见第 4 节）。对一个用 GitHub Releases 分发二进制的项目，**正确端点是 `github/v/release`**：它只反映真正发布出去的版本（本仓库实测 `/releases/latest` 存在，`tag_name=v0.3.0`、`prerelease=false`、`draft=false`），而 `v/tag` 会在"打了 tag 但没发 Release"时也显示版本，与发布事实脱钩。本仓库无 PyPI 包（PyPI 同名包为他人所有），GitHub Release 就是唯一的版本事实来源。

## 2. 颜色语义与实测

### 2.1 版本类徽章：何时橙、何时蓝

shields 源码 `services/color-formatters.js` 的 `version()` 函数（原文）：

```js
version = `${version}`
let first = version[0]
if (first === 'v') {
  first = version[1]
}
if (
  first === '0' ||
  /alpha|beta|snapshot|dev|pre|rc|scm|cvs/i.test(version)
) {
  return 'orange'
} else {
  return 'blue'
}
```

即：**版本号主版本为 0（v0.x），或串里含 alpha / beta / rc / pre / dev 等，一律橙色；v1.0.0 起变蓝**。这是 shields 有意为之的语义：0.x = 尚不稳定。来源：https://github.com/badges/shields/blob/master/services/color-formatters.js

调用链（`services/version.js` 的 `renderVersionBadge`）：`color: versionFormatter(isPrerelease ? 'pre' : version)`。所以：

- `include_prereleases` **本身不直接改颜色**，它只影响"哪个版本被选中"。选中预发布版本后会间接变橙，路径有二：`v/release` 会把 GitHub 标记的 prerelease 传成 `'pre'`（命中正则，橙）；`v/tag` 则靠版本串本身命中（如 `v1.0.0-rc1` 含 `rc`，橙）。
- 本仓库 v0.3.0 是正式版，实测加不加 `include_prereleases` 徽章完全相同（见 2.4）。

来源：https://github.com/badges/shields/blob/master/services/version.js

### 2.2 license 徽章：颜色按许可证类型映射

shields 源码 `services/licenses.js` 的映射表（节选）：

| 类别 | 颜色 | priority | AGPL 归属 |
| --- | --- | --- | --- |
| copyleft（要求"公开源码/同许可证"，含 GPL / LGPL / AGPL / MPL / EPL 全系） | `orange` | 1 | **AGPL-1.0/3.0（-only / -or-later 及别名 `AGPL-3.0`、`AGPLv3+`）全在 copyleft 列表** |
| permissive（MIT、Apache-2.0、BSD、ISC 等） | `green` | 2 | 否 |
| public domain（CC0、Unlicense、WTFPL、0BSD） | `7cd958` | 3 | 否 |
| 未映射 | `lightgrey` | 0 | 否 |

**AGPL-3.0 落在 copyleft → 橙色**。也就是说本仓库 license 徽章的橙色不是错误，而是 shields 有意的"强 copyleft 提示"（优先级压过 permissive 的绿）。来源：https://github.com/badges/shields/blob/master/services/licenses.js

### 2.3 命名色的实际色值

`badge-maker/lib/color.js`（右侧色块的实际十六进制值）：`orange = #ea7233`，`blue = #007ec6`，`green = #67ac09`，`brightgreen = #4b0`，`red = #dd4343`，`yellow = #d8b800`，`grey = #555`（左侧标签底色），`lightgrey = #939393`；另有别名（如 `informational → blue`）。与下文实测 SVG 中的 `#ea7233` / `#007ec6` 完全对上。来源：https://github.com/badges/shields/blob/master/badge-maker/lib/color.js

### 2.4 curl 实测（2026-10-31，img.shields.io）

| URL | 左标签 | 右侧文本 | 右侧填充色 |
| --- | --- | --- | --- |
| `/github/v/tag/ChHsiching/agent-web-search` | tag | v0.3.0 | `#ea7233` 橙 |
| `/github/v/tag/ChHsiching/agent-web-search?include_prereleases` | tag | v0.3.0 | `#ea7233` 橙（与上一条完全相同） |
| `/github/v/release/ChHsiching/agent-web-search` | release | v0.3.0 | `#ea7233` 橙 |
| `/github/license/ChHsiching/agent-web-search` | license | AGPL-3.0 | `#ea7233` 橙 |
| `/github/v/release/ChHsiching/agent-web-search?label=version` | version | v0.3.0 | `#ea7233` 橙（只改标签不改色） |
| `/github/v/release/ChHsiching/agent-web-search?label=version&color=blue` | version | v0.3.0 | `#007ec6` 蓝 |
| `/github/license/ChHsiching/agent-web-search?color=blue` | license | AGPL-3.0 | `#007ec6` 蓝 |
| `/badge/License-AGPL--3.0-blue` | License | AGPL-3.0 | `#007ec6` 蓝 |
| `/badge/license-AGPL%20v3-blue.svg` | license | AGPL v3 | `#007ec6` 蓝 |
| `/github/license/grafana/grafana`（对照） | license | AGPL-3.0 | `#ea7233` 橙（grafana 保持默认） |

关键行原样摘录：

```text
# v/tag（当前 README 用的端点）
<title>tag: v0.3.0</title>            aria-label="tag: v0.3.0"
rect: fill="#555"（左侧） fill="#ea7233"（右侧）

# v/release
<title>release: v0.3.0</title>        aria-label="release: v0.3.0"
rect: fill="#555" fill="#ea7233"

# license
<title>license: AGPL-3.0</title>      aria-label="license: AGPL-3.0"
rect: fill="#555" fill="#ea7233"

# 推荐写法实测：?label=version&color=blue
<title>version: v0.3.0</title>        aria-label="version: v0.3.0"
rect: fill="#555" fill="#007ec6"

# 静态徽章 /badge/License-AGPL--3.0-blue
<title>License: AGPL-3.0</title>
rect: fill="#555" fill="#007ec6"
```

**结论 2**：两枚徽章都是橙色，原因各不相同且都是 shields 的既定语义：版本徽章橙是因为 v0.3.0 主版本为 0；license 徽章橙是因为 AGPL 属 copyleft。`include_prereleases` 对本仓库的颜色和内容均无影响。

## 3. license 徽章的数据链路

1. shields 的 `github/license` 服务请求 `GET /repos/{user}/{repo}`，取返回体里的 `license.spdx_id` 作为右侧文本。源码 `services/github/github-license.service.js`：`spdx_id === 'NOASSERTION'` 时显示 "not identifiable by github"；`license === null` 时显示 "not specified"；否则 `renderLicenseBadge({ license })`（文本 = spdx_id，颜色按第 2.2 节映射）。来源：https://github.com/badges/shields/blob/master/services/github/github-license.service.js
2. GitHub API 文档：`license` 字段是 `License Simple` 对象（`spdx_id`: string or null），随仓库接口返回。来源：https://docs.github.com/en/rest/repos/repos#get-a-repository
3. 这个字段由 GitHub 用 **licensee** 生成：读 LICENSE 文件，先精确匹配（去空白与版权行后直接字符串比较），再用 Sørensen–Dice 相似度匹配内置许可证库。AGPL-3.0 属于 licensee 内置许可证，**标准文本可以被自动识别**。来源：https://github.com/licensee/licensee（README "The solution" 一节）
4. 本仓库实测（`gh api repos/ChHsiching/agent-web-search --jq '.license'`，2026-10-31）：

```json
{"key":"agpl-3.0","name":"GNU Affero General Public License v3.0","node_id":"MDc6TGljZW5zZTE=","spdx_id":"AGPL-3.0","url":"https://api.github.com/licenses/agpl-3.0"}
```

即徽章显示的 "AGPL-3.0" 正是 GitHub 识别出的 spdx_id。注意 shields 显示的是 GitHub 归一化后的 `AGPL-3.0`（对应 SPDX 的 AGPL-3.0-only/or-later 家族），不是 `AGPL-3.0-only` 字面量。

## 4. 控制 / 覆盖外观的正规方式

### 4.1 查询参数（shields.io 官方文档，对所有徽章通用）

来源：https://shields.io/badges/static-badge （Static badge 构建器页，"Query Parameters" 一节，原文摘录）

| 参数 | 官方说明（原文） | 备注 |
| --- | --- | --- |
| `label` | "Override the default left-hand-side text (URL-Encoding needed for spaces or special characters!)" | 解决抱怨①的正解；空格等需 URL 编码 |
| `labelColor` | "Background color of the left part (hex, rgb, rgba, hsl, hsla and css named colors supported)." | 左侧底色，默认 `#555` |
| `color` | "Background color of the right part (hex, rgb, rgba, hsl, hsla and css named colors supported)." | 支持**命名色与十六进制**（也接受 rgb/rgba/hsl/hsla）。命名色以 shields 自己的表为准，"Some named colors may differ from CSS color values"，完整列表见 badge-maker README（https://github.com/badges/shields/blob/master/badge-maker/README.md）。实测 `color=blue` 覆盖语义橙为 `#007ec6` |
| `link` | "Specify what clicking on the left/right of a badge should do. Note that this only works when integrating your badge in an `<object>` HTML tag, but not an `<img>` tag or a markup language." | **在 markdown README 里无效**。README 中的点击目标必须用 markdown 图片链接 `[![alt](img)](url)` 包裹实现（现行写法已如此，保持即可） |

### 4.2 静态徽章 `/badge/` 方案

语法：`https://img.shields.io/badge/<label>-<message>-<color>`，三段用 `-` 分隔；文字里的连字符要写成 `--` 转义，空格用 `%20` 或 `_`。实测：

- `https://img.shields.io/badge/License-AGPL--3.0-blue` → `License | AGPL-3.0` 蓝（`--` 转义出 `AGPL-3.0` 里的连字符）
- `https://img.shields.io/badge/license-AGPL%20v3-blue.svg` → `license | AGPL v3` 蓝（`%20` 空格；URL 结尾的 `.svg` 后缀可留可去）
- 下划线写法：`License-AGPL_v3-blue`（immich 在用，见第 5 节）

利弊：

- 利：完全可控、渲染恒定；不依赖 GitHub 的许可证识别和 release 状态；可随意配 logo、style。
- 弊：**不自动跟随**。版本徽章用静态意味着每次发版都要手改 README，不可接受；license 徽章用静态则在许可证变更时要记得手改，且徽章内容与仓库事实脱钩（写错无任何告警）。

来源：https://shields.io/badges/static-badge （"Path Parameters: Label, (optional) message, and color. Separated by dashes"）；转义规则见 https://github.com/badges/shields/blob/master/badge-maker/README.md

## 5. 高星 AGPL-3.0 项目的 License 徽章惯例（2026-10-31 读 raw README 源码）

| 项目（星数） | 写法原文 | 路数 |
| --- | --- | --- |
| immich-app/immich（117,790） | `<a href="https://opensource.org/license/agpl-v3"><img src="https://img.shields.io/badge/License-AGPL_v3-blue.svg?color=3F51B5&style=for-the-badge&label=License&logoColor=000000&labelColor=ececec" alt="License: AGPLv3"></a>` | **静态徽章**，蓝系自定义色（`3F51B5`），`_` 代空格，链接指向 opensource.org 的 AGPL 页 |
| grafana/grafana（77,597） | `[![License](https://img.shields.io/github/license/grafana/grafana)](LICENSE)` | **动态徽章，保持默认**（实测渲染 `license | AGPL-3.0`，橙色 `#ea7233`，与本项目现状相同） |
| nextcloud/server（37,371） | README 顶部无 shields license 徽章，用的是 REUSE 合规徽章：`[![REUSE status](https://api.reuse.software/badge/github.com/nextcloud/server)](...)` | 第三方合规徽章 |
| （补充观察）misskey-dev/misskey（11,367）、element-hq/element-web（13,635） | README 顶部均无 license 徽章 | 无惯例 |

来源：raw.githubusercontent.com 各仓库 README（immich main、grafana main、nextcloud master、misskey develop、element-web develop），星数与 license 取自 api.github.com（均为 AGPL-3.0）。

**结论 5**：AGPL 高星项目没有统一惯例，主流只有两派：动态默认（grafana，接受橙色语义）或静态自定义蓝（immich）。把动态徽章覆盖成蓝色不属于标新立异。

## 6. 最终结论：本仓库推荐写法

针对本仓库（GitHub Releases 分发二进制、AGPL-3.0-only、无 PyPI）：

### 6.1 版本徽章（推荐）

```markdown
[![GitHub release](https://img.shields.io/github/v/release/ChHsiching/agent-web-search?label=version&color=blue)](https://github.com/ChHsiching/agent-web-search/releases/latest)
```

理由：

1. **端点换成 `github/v/release`**：仓库以 Releases 分发，版本事实来源是 Release 而非 git tag（第 1 节）。
2. **`label=version`**：`v/release` 默认标签是 "release"（`v/tag` 是 "tag"），两者都不会自己显示 "version"；`label` 是官方覆盖参数（第 4.1 节）。解决抱怨①。
3. **`color=blue`**：消除 0.x 期的语义橙，解决抱怨②。若更看重 shields 的"pre-1.0 不稳定"提示语义，可去掉该参数：v1.0.0 发布之日起自动变蓝，零维护。fetch 兄弟仓库 2026-10-31 已落地 `color=blue`（见 6.4），此处保持一致并补上 `label=version`。
4. **去掉 `include_prereleases` 与 `sort=semver`**：实测最新 release v0.3.0 是正式版，默认走 `/releases/latest` 即正确。这两个参数 `v/release` 同样支持，但只在需要让预发布参选（`include_prereleases`）或按语义版本而非发布时间取最新（`sort=semver`）时才有意义；参数越少越不容易误读。

### 6.2 license 徽章（推荐，保持动态）

```markdown
[![License: AGPL-3.0](https://img.shields.io/github/license/ChHsiching/agent-web-search?color=blue)](https://github.com/ChHsiching/agent-web-search/blob/main/LICENSE)
```

理由：

1. **保持动态端点**：GitHub licensee 已正确识别本仓库许可证（实测 spdx_id=AGPL-3.0，第 3 节），徽章文本自动跟随，零维护；许可证真变了徽章自己会变。
2. **`color=blue`**：橙色是 shields 对 copyleft 的刻意提示而非故障（第 2.2 节）；覆盖成中性蓝是纯观感选择，immich 等高星 AGPL 项目同样蓝系（第 5 节）。想保留"copyleft 提示"语义就去掉该参数。
3. alt 文字写成 `License: AGPL-3.0`，与徽章内容一致。

备选（静态）：`[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue)](https://www.gnu.org/licenses/agpl-3.0)`。写法可控（immich 同款），但许可证变更需手动同步，对本仓库没有额外收益，列为次选。

### 6.3 实测渲染效果（即 6.1 / 6.2 落地后的样子）

`version | v0.3.0`（右块 `#007ec6`）与 `license | AGPL-3.0`（右块 `#007ec6`），左块均为默认 `#555`。两枚徽章同蓝、标签一 "version" 一 "license"，两个抱怨同时解决。

### 6.4 落地范围提示

- 同样的两行存在于 `README.zh-CN.md` 第 4-5 行，应同步修改。
- fetch 兄弟仓库（`C:\Users\Administrator\Git\agent-web-fetch`）README 第 4-5 行已在 2026-10-31 改为 `github/v/release?color=blue` 与 `github/license?color=blue`，但版本徽章**未加 `label=version`**，左标签仍显示 "release"；如需与本报告结论对齐，把 `label=version` 一并补上即可（换 repo 名）。
- 徽章点击目标继续用 markdown `[![…]](…)` 包裹实现，不要改用 `link` 查询参数（markdown 中无效，第 4.1 节）。

## 7. 来源清单

shields 源码（均为 badges/shields master 分支，2026-10-31 取自 raw.githubusercontent.com）：

- https://github.com/badges/shields/blob/master/services/github/github-tag.service.js
- https://github.com/badges/shields/blob/master/services/github/github-release.service.js
- https://github.com/badges/shields/blob/master/services/github/github-common-release.js
- https://github.com/badges/shields/blob/master/services/github/github-license.service.js
- https://github.com/badges/shields/blob/master/services/version.js
- https://github.com/badges/shields/blob/master/services/color-formatters.js
- https://github.com/badges/shields/blob/master/services/licenses.js
- https://github.com/badges/shields/blob/master/badge-maker/lib/color.js
- https://github.com/badges/shields/blob/master/badge-maker/README.md

shields.io 官方文档页：

- https://shields.io/badges/static-badge （label / labelColor / color / link 参数与 /badge/ 语法）
- https://shields.io/badges/git-hub-release 、https://shields.io/badges/git-hub-tag 、https://shields.io/badges/git-hub-license

GitHub / licensee：

- https://docs.github.com/en/rest/repos/repos#get-a-repository （license 字段与 License Simple / spdx_id）
- https://github.com/licensee/licensee （许可证识别机制）
- 实测：`GET https://api.github.com/repos/ChHsiching/agent-web-search`（license 对象）、`GET .../releases/latest`（v0.3.0，非 prerelease）

高星 AGPL 项目 README（raw.githubusercontent.com，2026-10-31）：

- https://raw.githubusercontent.com/immich-app/immich/main/README.md
- https://raw.githubusercontent.com/grafana/grafana/main/README.md
- https://raw.githubusercontent.com/nextcloud/server/master/README.md
- https://raw.githubusercontent.com/misskey-dev/misskey/develop/README.md
- https://raw.githubusercontent.com/element-hq/element-web/develop/README.md
