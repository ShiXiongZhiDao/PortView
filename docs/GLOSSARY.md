# PortLite 术语表（Glossary）

| 术语 | 定义 |
|---|---|
| 连接（Connection） | 一条 TCP 连接或 UDP 端点记录，由协议、端口、地址、状态、属主进程构成 |
| TCP 连接总数 | 所有 TCP 连接条目数，含 LISTENING（统计口径） |
| UDP 端点 | 一条 UDP 绑定记录；UDP 无连接状态，状态列显示 `-` |
| 监听端口（LISTENING） | TCP 处于监听状态的条目数（统计口径） |
| 进程数 | 系统全部进程数量（合并视图统计口径，含无网络连接进程） |
| PID | 进程标识符（Process ID） |
| 本地地址/远程地址 | 格式 `IP:端口`；远程地址为 `0.0.0.0` 表示任意远端；UDP 无远程地址 |
| 状态（TCP） | LISTENING / ESTABLISHED / TIME_WAIT / CLOSE_WAIT / SYN_SENT 等 TCP 状态 |
| 软件名 | 从进程 exe 版本资源读取的 `FileDescription` 或 `ProductName` |
| 拼音索引 | 中文软件名/进程名经 pinyin-pro 生成的全拼与首字母，供搜索命中 |
| 模糊搜索 | 子串部分匹配（非精确匹配），对端口/进程名/PID/路径/软件名/拼音任一字段命中即显示 |
| 刷新 | 点击"刷新"按钮重新调用 Rust 采集接口获取最新进程与连接快照；"结束进程"完成后自动刷新一次 |
| 结束进程 | 对行内 PID 调用 `taskkill /F` 终止进程，执行前需确认 |
| 合并视图 | 进程与连接合并在同一表格：每行一个进程（含端口摘要），行展开显示该进程全部连接明细 |
| requireAdministrator | Windows 清单提权声明，应用以管理员权限运行 |
