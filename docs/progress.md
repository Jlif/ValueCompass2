# 当前进展

**更新时间**: 2026-03-26

## 进行中

- 🔄 技术架构设计文档编写（100%）
  - Tauri + React + Python/AKTools 架构确定
  - 数据存储策略：SQLite 存股票列表，K线实时拉取

## 待办

- [ ] 初始化 Tauri 项目结构
- [ ] 配置前端 React + TypeScript 环境
- [ ] 编写 Python 服务打包脚本
- [ ] 设计 SQLite 表结构并实现 Rust 端操作
- [ ] 集成 AKTools HTTP 客户端
- [ ] 实现股票列表展示页面
- [ ] 集成 lightweight-charts K线图

## 已完成

- [x] 技术选型决策（Tauri vs Electron vs Flutter）
- [x] 数据架构设计（本地存储 + 实时拉取策略）
- [x] 技术架构设计文档

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

初始化项目代码结构，开始编写实现代码。
