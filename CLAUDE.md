# ValueCompass2

## 项目概述

项目名称：ValueCompass2
技术栈：Tauri v2 + React 19 + TypeScript + Rust + Python/AKTools
项目描述：A股价值分析桌面应用 - 基于 AKShare 的股票数据分析工具

## 目录结构

```
CLAUDE.md              # 项目专属信息（技术栈、命令、架构）
docs/
├── README.md          # 文档索引
├── progress.md        # 当前快照（在做什么、下一步）
├── memory/            # 工作日志
│   └── README.md
├── requirements/      # 需求文档
│   └── README.md
├── features/          # 功能文档
│   └── README.md
└── postmortem/        # 复盘记录
    └── README.md
src/                   # 前端 React 源码
├── App.tsx
├── main.tsx
└── ...
src-tauri/             # Tauri Rust 后端
├── src/               # Rust 源码
├── Cargo.toml         # Rust 依赖
└── tauri.conf.json    # Tauri 配置
public/                # 静态资源
index.html             # 入口 HTML
package.json           # npm 依赖
vite.config.ts         # Vite 配置
tsconfig.json          # TypeScript 配置
```

## 工作流命令

| 命令 | 用途 |
|------|------|
| `npm run dev` | 启动前端开发服务器 |
| `npm run tauri dev` | 启动 Tauri 开发模式（完整应用） |
| `npm run tauri build` | 构建生产版本 |
| `/checkpoint` | 存档当前进展到 memory 和 progress |
| `/recap` | 恢复上下文，读取最近的 memory 和 progress |
| `/postmortem` | 复盘踩坑经过，沉淀教训 |

## 技术要点

- **前端框架**: React 19 + TypeScript + Vite
- **桌面端**: Tauri v2 (Rust)
- **数据源**: AKTools (Python/AKShare HTTP API)
- **本地存储**: SQLite (rusqlite)
- **图表库**: lightweight-charts (待集成)
