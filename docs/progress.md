# 当前进展

**更新时间**: 2026-03-27
**Git Commit**: 8c956db

## 已完成

### 后端 (Rust)
- [x] 初始化 Tauri 项目结构
- [x] SQLite 数据库模块（rusqlite）- stocks/watchlist/sync_log 表
- [x] Python 服务管理模块 - 启动/停止/健康检查
- [x] AKTools HTTP 客户端
  - 股票列表: `stock_info_a_code_name`
  - K 线数据: `stock_zh_a_daily`
- [x] Tauri 命令暴露 - 数据库/服务管理/数据获取
- [x] 启动时自动同步（数据库为空时自动同步股票列表）

### 前端 (React + TypeScript)
- [x] 股票列表展示页面 (`src/App.tsx`)
- [x] 搜索功能
- [x] 同步股票列表功能
- [x] K 线数据表格展示（日期格式化为 YYYY-MM-DD）
- [x] 服务状态监控

### 打包
- [x] PyInstaller 打包脚本 (`python/build.py`)
- [x] 占位文件解决 Tauri sidecar 编译问题

### 踩坑记录
- [x] POST-001: Tauri + Python 混合架构踩坑记录

## 已修复问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 代理导致东方财富 API 无法连接 | 系统代理设置 | 切换可用接口 (`stock_info_a_code_name`, `stock_zh_a_daily`) |
| Tauri sidecar 编译错误 | 缺少占位文件 | 创建占位文件 `bin/aktools-aarch64-apple-darwin` |
| API 404 错误 | URL 缺少 `/public/` 路径 | 修正 URL: `/api/public/stock_info_a_code_name` |
| lightweight-charts v5 API 变更 | 版本升级 | 使用新 API (`addSeries(CandlestickSeries, ...)`) |
| K 线日期显示格式 | 返回 ISO 格式 | 前端截取 `item.date.split('T')[0]` |

## 运行方式

### 开发模式
```bash
# 1. 启动 aktools（手动，因为占位文件无法执行）
unset http_proxy https_proxy ALL_PROXY
python -m aktools --port=8080 --host=127.0.0.1 &

# 2. 启动 Tauri 开发模式
npm run tauri dev
```

### 使用说明
1. 应用启动后自动检测数据库，如为空则自动同步股票列表
2. 左侧列表显示所有 A 股（5493 只）
3. 搜索框可按代码或名称搜索
4. 点击股票名称查看 K 线数据

## 下一步

- [ ] lightweight-charts 图表展示优化
- [ ] 生产构建测试（PyInstaller 打包 Python 服务）
- [ ] 完善错误处理和日志记录
