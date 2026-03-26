# Python 服务打包

此目录包含将 AKTools 打包为独立可执行文件的脚本，用于随 Tauri 应用一起分发。

## 文件说明

| 文件 | 说明 |
|------|------|
| `requirements.txt` | Python 依赖列表 |
| `aktools_server.py` | 入口包装脚本，处理启动参数和信号 |
| `build.py` | PyInstaller 打包脚本 |
| `README.md` | 本文档 |

## 快速开始

### 1. 安装依赖

```bash
cd python
pip install -r requirements.txt
```

### 2. 打包

```bash
# 默认打包到 ../src-tauri/bin/macos (或 windows/linux)
python build.py

# 打包为单文件（启动稍慢，体积小）
python build.py --onefile

# 指定输出目录
python build.py --output-dir=/path/to/output

# 清理缓存后打包
python build.py --clean
```

### 3. 验证

```bash
# 测试打包结果
../src-tauri/bin/macos/aktools --port=8080

# 访问健康检查
curl http://127.0.0.1:8080/
```

## 输出结构

### 目录模式（默认）
```
src-tauri/bin/macos/
├── aktools              # 主可执行文件
├── _internal/           # 依赖库
└── ...
```

### 单文件模式
```
src-tauri/bin/macos/
└── aktools              # 单个可执行文件
```

## 与 Tauri 集成

Rust 代码会自动查找以下位置的 aktools 可执行文件：

1. 应用包内（打包后）
   - macOS: `../Resources/python/aktools`
   - Windows: 同级目录或 `python/aktools.exe`
   - Linux: 同级目录或 `python/aktools`

2. 开发环境
   - 系统 PATH 中的 `aktools`

## 平台差异

| 平台 | 输出目录 | 可执行文件名 |
|------|----------|--------------|
| macOS | `src-tauri/bin/macos/` | `aktools` |
| Windows | `src-tauri/bin/windows/` | `aktools.exe` |
| Linux | `src-tauri/bin/linux/` | `aktools` |

## 注意事项

1. **首次打包耗时**: 需要下载并打包所有依赖，约 5-10 分钟
2. **体积**: 输出约 200-400MB（包含 Python 运行时和 AKShare）
3. **增量构建**: 删除 `build/` 和 `dist/` 目录可强制重新打包
4. **跨平台**: 必须在目标平台上打包（PyInstaller 不支持交叉编译）

## CI/CD 集成

GitHub Actions 示例：

```yaml
- name: Build Python Service
  run: |
    cd python
    pip install -r requirements.txt
    python build.py

- name: Build Tauri
  run: npm run tauri build
```
