# 当前进展

**更新时间**: 2026-03-27 17:30
**Git Commit**: 待提交

## 已完成

### 后端 (Rust)
- [x] 初始化 Tauri 项目结构
- [x] SQLite 数据库模块（rusqlite）
- [x] Python 服务管理模块
  - 动态端口分配（自动查找可用端口）
  - 服务健康检测（使用 `/version` 端点）
  - 启动/停止服务
  - 检测已存在服务并复用
  - 启动超时延长到 60 秒
- [x] AKTools HTTP 客户端
  - 修复 K 线 API 500 错误（切换为 stock_zh_a_hist）
  - 日期格式自动转换
- [x] Tauri 命令暴露

### 前端 (React + TypeScript)
- [x] 股票列表展示页面
- [x] 搜索功能
- [x] 同步股票列表功能
- [x] K 线数据展示（lightweight-charts）
- [x] 服务状态监控
  - 动态按钮：启动/停止/启动中...
  - 自适应轮询频率：1s（变化中）/ 10s（稳定）

### 打包
- [x] PyInstaller 打包脚本（单文件模式）
- [x] Tauri sidecar 配置
- [x] 生产构建测试通过

### 踩坑记录
- [x] POST-001: Tauri + Python 混合架构踩坑记录
- [x] POST-002: AKTools API 404 错误与数据同步问题
- [x] POST-003: Tauri + AKTools 打包调试复盘

## 已修复问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| Python 服务启动失败 | `get_default_application` 不存在 | 改用动态导入 `aktools.main` |
| 端口冲突 | 8080 被占用 | 动态端口分配（18080+） |
| 重复启动服务 | 根路径 `/` 返回 500 | 改用 `/version` 端点检测 |
| 服务无法停止 | 无进程句柄 | 通过 `lsof` + `kill` 停止 |
| 按钮状态混乱 | 无禁用状态 | 添加 `Starting` 状态禁用 |
| K 线 500 错误 | stock_zh_a_daily 不稳定 | 切换为 stock_zh_a_hist |
| 日期格式错误 | API 返回 ISO 8601 | Rust 层截取日期部分 |

## 应用包位置

```
src-tauri/target/release/bundle/macos/ValueCompass2.app
```

## 功能验证

- ✅ 应用启动自动检测/启动 Python 服务
- ✅ 5493 只 A 股列表展示
- ✅ 搜索功能（代码/名称）
- ✅ 股票详情 + K 线图表
- ✅ 手动启动/停止服务
- ✅ 服务状态实时显示
- ✅ K 线数据正常显示

## 下一步

- [ ] UI 美化
- [ ] 数据库持久化路径优化
- [ ] 添加更多技术指标
