# port-view Design System (MASTER)

> 由 UI/UX Pro Max 技能指导，按 **LearnHub（educational-platform）** 风格重绘：
> 参考 `https://uupm.cc/demo/educational-platform`（Flat & Clean：米白底 + 白卡 + 轻阴影 + 鲜绿主色 + pill 标签/按钮）。
> 适用范围：port-view 桌面应用全部页面。页面级覆盖见 `pages/`（当前无覆盖）。

## 风格（Style）

- **Flat & Clean（扁平清新）**：纯白卡片 + 轻阴影（非黏土重阴影）、圆角 14-18px、pill（胶囊）标签与按钮。
- **LearnHub 视觉语言**：米白背景衬托白色卡片；浅色底 + 深色字的 pill 标签（浅绿/浅蓝）；绿色实心主按钮。
- 工具产品约束：表格信息密度保持紧凑，正文对比度 ≥ 4.5:1；不做重动效。

## 色彩（Colors）

| Token | 亮色 | 暗色 | 用途 |
|---|---|---|---|
| --pv-bg | #F7F2E8 | #0F172A | 页面背景（米白 / 深蓝灰） |
| --pv-card | #FFFFFF | #1E293B | 白卡（标题栏/统计卡/工具栏/表格容器） |
| --pv-card-2 | #FAF7F0 | #172033 | 次级面（展开行、禁用输入） |
| --pv-border | #EFE9DC | #2B3648 | 极浅边框 |
| --pv-text | #1F2937 | #F1F5F9 | 标题/主文本 |
| --pv-text-soft | #6B7280 | #94A3B8 | 正文灰 |
| --pv-green | #16A34A | #22C55E | 主色/CTA（刷新按钮、logo、绿标签字底） |
| --pv-green-bg | #DCFCE7 | #14532D | 浅绿 pill 标签底 |
| --pv-green-text | #166534 | #86EFAC | 深绿标签字 |
| --pv-blue-bg / text | #DBEAFE / #1D4ED8 | #1E3A5F / #93C5FD | 浅蓝（UDP 图标块、次按钮） |
| --pv-amber-bg / text | #FEF3C7 / #92400E | #3A3220 / #FCD34D | 奶油（监听端口图标块） |
| --pv-violet-bg / text | #EDE9FE / #5B21B6 | #2E2A45 / #C4B5FD | 淡紫（进程图标块） |

## 字体（Typography）

- 标题/数字：**Fredoka**（本地 woff2，weight 300-700）
- 正文/界面：**Nunito**（本地 woff2，weight 300-900）
- 中文回退：Segoe UI / Microsoft YaHei

## 形状与阴影（Shape & Shadow）

- 大圆角：`--pv-radius-lg: 18px`（卡片）
- 中圆角：`--pv-radius-md: 14px`
- 小圆角：`--pv-radius-sm: 10px`（输入、图标块）
- pill：`--pv-radius-pill: 999px`（按钮、标签）
- 卡片阴影：`0 2px 8px rgba(31,41,55,.05), 0 8px 24px rgba(31,41,55,.06)`（暗色对应变体）
- 按压：`translateY(1px)` + 阴影收窄

## 布局规范（Layout）

- **Hero 区**：贴纸白卡（3px 深色边框 + 4px 偏移阴影）；左侧 56px 绿色圆角 logo（同边框）+ Fredoka 40px 大标题 + 灰色描述；右侧大搜索框 + 主题下拉 + 绿色 pill 刷新按钮；底部分隔线 + 4 项 Fredoka 32px 数字统计条（TCP/UDP/监听/进程）
- **筛选条**："筛选" 标签 + 协议下拉 + 状态下拉（均 3px 深色边框）
- **进程列表**：单个贴纸卡片容器（3px 边框 + 偏移阴影），内部 sticky 表头（进程/PID/TCP/UDP/监听/端口/状态/操作，浅米底 + 3px 下边框）；每行一个进程（36px 彩色图标块 + 进程名/路径 + 各列数字 + 端口 pill + Active/Idle pill + 连接/结束按钮），行间虚线分隔；点击"连接"在行下方展开连接明细（每条连接一个小贴纸条：协议/端口/本地/远程/状态 pill）；同时只展开一个进程；无分页，容器内滚动
- **空状态**：居中浅蓝圆角块（3px 边框 + 偏移阴影）+ ∅ 符号 + "没有匹配的进程" + 提示文字
- **贴纸边框体系**：所有方框统一 3px 深色粗边框（--pv-stroke，亮色 #1F2937 / 暗色 #CBD5E1）+ 同色偏移硬阴影（卡片 4px、按钮 3px）；按压时 translate(2px,2px) + 阴影缩小到 1px

## 组件要点（Components）

- 标题栏：白卡横条；44px 绿色圆角 logo（PV，带绿色柔光投影）；标题 Fredoka 粗体 + "Windows Port Watcher" 浅绿 pill 小标签
- 统计卡：白卡 + 圆角 18px + 轻阴影；左侧 48px 彩色圆角图标块（T/U/L/P，浅底深字）；Fredoka 大数字
- 工具栏 / 表格容器：白卡 + 轻阴影
- 按钮：pill 圆角、字重 800；刷新=绿色实心；结束进程=红色
- 表格：表头浅米（亮）/深灰蓝（暗）、行 hover 绿色微光、展开行背景 pv-card-2
- pill 标签：浅色底 + 深色字 + 全圆角

## 反模式（Anti-patterns）

- 不使用 emoji 作图标
- 浅色模式正文对比度 ≥ 4.5:1
- 统计卡颜色不能只靠色相区分（附文字标签）
- 不用重黏土阴影 / 厚边框（与 LearnHub 扁平语言冲突）
- 桌面工具避免大面积动效
