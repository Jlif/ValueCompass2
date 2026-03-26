# POST-001: Tauri + Python 混合架构踩坑记录

## 时间线

- 2026-03-26 上午 - 确定技术架构（Tauri + React + Python/AKTools）
- 2026-03-26 下午 - 实现 Python 服务管理模块
- 2026-03-26 晚上 - 配置 Tauri externalBin 时遇到编译错误
- 2026-03-26 晚上 - 创建占位文件解决编译问题

## 现象

配置 Tauri `externalBin` 后编译报错：
```
resource path `bin/aktools-aarch64-apple-darwin` doesn't exist
```

## 根因分析

### 表面原因
Tauri 在编译时检查 `externalBin` 配置的文件是否存在，文件不存在导致编译失败。

### 真正原因
Tauri 的 sidecar 机制要求二进制文件在编译时就存在，但：
1. PyInstaller 打包耗时很长（5-10分钟）
2. 开发阶段不需要真实的 Python 服务（可以手动启动 aktools）
3. 两者存在时间上的矛盾

## 解决方案

创建占位脚本绕过编译检查：

```bash
mkdir -p src-tauri/bin
echo '#!/bin/bash
echo "AKTools not built yet. Run: cd python && python build.py"' \
  > src-tauri/bin/aktools-aarch64-apple-darwin
chmod +x src-tauri/bin/aktools-aarch64-apple-darwin
```

开发流程：
1. 开发阶段使用占位文件，手动启动 `aktools --port=8080`
2. 生产构建前执行 `python build.py` 生成真实二进制文件
3. Tauri 打包时自动包含 sidecar

## 架构决策记录

### 为什么选择 Tauri + Python 而不是纯 Tauri？

| 方案 | 优点 | 缺点 |
|------|------|------|
| 纯 Tauri (Rust) | 单语言，无 sidecar 复杂性 | AKShare 是 Python 独占库，无法直接调用 |
| Tauri + Python sidecar | 保留 AKShare 能力 | sidecar 管理复杂，体积大 (~200MB) |
| Electron + Node.js | 成熟方案 | 体积巨大 (150MB+)，内存占用高 |

最终选择方案2，因为 AKShare 的数据源能力无法替代。

### 数据存储策略决策

**问题**: A股10年K线数据约 2-3GB，如何存储？

| 方案 | 优点 | 缺点 |
|------|------|------|
| 全量本地存储 | 完全离线可用 | 首次同步2-3GB，用户体验差 |
| 纯在线获取 | 无本地存储压力 | 完全依赖网络，无法离线查看 |
| **轻本地 + 实时拉取** | 首次启动快，按需加载 | 需要网络，历史数据需等待 |

选择方案3：SQLite 仅存股票列表（~5000条），K线实时拉取+内存缓存。

## 教训

1. **Tauri sidecar 的编译时检查**
   - externalBin 配置的文件必须在编译时存在
   - 可以用占位文件绕过，但要注意平台命名规范（`{bin}-{target-triple}`）

2. **PyInstaller 打包的时间成本**
   - 首次打包 5-10 分钟， CI/CD 需要考虑缓存策略
   - 目录模式（onedir）比单文件（onefile）启动更快

3. **混合架构的服务管理**
   - Python 服务启动需要 2-5 秒，需要加载状态提示
   - 应用关闭时必须确保 Python 进程被终止（Drop trait）

4. **AKTools 的局限性**
   - 依赖东方财富接口，存在隐性限流
   - 需要实现防抖和错误重试机制
   - 不能用于高频交易场景

## 后续优化方向

1. 实现 Python 服务预打包脚本，开发时也使用真实 sidecar
2. 添加 K 线数据本地缓存（SQLite 或文件缓存）
3. 实现请求队列和防抖，避免触发东方财富限流
