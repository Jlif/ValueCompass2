# 当前进展

**更新时间**: 2026-03-26

## 进行中

- 🔄 编写 Python 服务打包脚本（进行中）
  - AKTools 进程管理（启动/停止/健康检查）
  - 服务状态监控

## 待办

- [x] 初始化 Tauri 项目结构
- [x] 配置前端 React + TypeScript 环境
- [x] 设计 SQLite 表结构并实现 Rust 端操作
  - stocks 表（股票基础信息）
  - watchlist 表（用户自选）
  - sync_log 表（同步日志）
- [x] 集成 AKTools HTTP 客户端
  - 股票列表获取
  - K 线数据获取
  - 个股详细信息
- [ ] 编写 Python 服务打包脚本（PyInstaller）
- [ ] 实现股票列表展示页面
- [ ] 集成 lightweight-charts K线图

## 已完成

- [x] 技术选型决策（Tauri vs Electron vs Flutter）
- [x] 数据架构设计（本地存储 + 实时拉取策略）
- [x] 技术架构设计文档
- [x] 项目初始化（Tauri + React + TypeScript）
- [x] SQLite 数据库模块（rusqlite）
- [x] Python 服务管理模块
- [x] AKTools HTTP 客户端

## 关键决策

1. **桌面端技术栈**: Tauri + React + TypeScript
   - 理由：安装包小、内存占用低、Rust 后端性能好

2. **数据源方案**: AKTools 提供 HTTP 接口
   - 理由：封装 AKShare，无需重复开发爬虫

3. **数据存储策略**: SQLite 仅存股票列表（~5000条），K线实时拉取+内存缓存
   - 理由：避免首次同步大量数据，平衡离线可用性

4. **Python 服务管理**: 内嵌打包，随 Tauri 启动/关闭
   - 理由：用户无感知，单应用体验

## 卡住的地方

无

## 下一步

1. 编写 Python 服务打包脚本（PyInstaller 打包 AKTools）
2. 实现前端股票列表展示页面
3. 集成 lightweight-charts 显示 K 线图
