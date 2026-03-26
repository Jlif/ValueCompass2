# POST-002: AKTools API 404 错误与数据同步问题

## 时间线

- 23:20 - 用户反馈：点击"同步股票列表"界面无变化
- 23:23 - 发现两个 aktools 进程占用端口
- 23:28 - 查看日志发现 "HTTP error: 404 Not Found"
- 23:30 - 对比日志发现 Rust 代码缺少 `/public/` 路径前缀
- 23:32 - 修复 URL 路径，重新构建测试
- 23:35 - 股票列表同步成功
- 23:37 - 用户反馈 K 线日期格式需要处理
- 23:38 - 前端添加日期格式化代码

## 现象

1. 点击"同步股票列表"按钮后，界面不显示股票数据
2. 后端日志显示 "HTTP error: 404 Not Found"
3. K 线日期显示为 `2026-03-26T00:00:00.000` 格式

## 根因分析

### 问题1：API 404 错误

#### 表面原因
- Rust 代码调用 AKTools API 返回 404
- 对比日志发现手动 curl 成功但 Rust 代码失败

#### 真正原因
- Rust 代码使用 `/api/stock_info_a_code_name`
- 正确路径应该是 `/api/public/stock_info_a_code_name`
- 同样问题存在于 K 线 API `/api/stock_zh_a_daily` vs `/api/public/stock_zh_a_daily`

**为什么？**
- 前期测试时直接测试了 `/api/public/xxx` 路径
- 但修改 Rust 代码时遗漏了 `/public/` 前缀
- 代码审查时未注意到路径不一致

### 问题2：K 线日期格式

- AKShare `stock_zh_a_daily` 接口返回 ISO 8601 格式日期
- 前端直接显示原始字符串 `2026-03-26T00:00:00.000`
- 用户期望 `2026-03-26` 格式

## 解决方案

### 修复 404 错误

```rust
// 股票列表 API
let url = format!("{}/api/public/stock_info_a_code_name", self.base_url);

// K 线 API
let url = format!("{}/api/public/stock_zh_a_daily", self.base_url);
```

### 修复日期格式

```tsx
// 前端截取日期部分
<td>{item.date.split('T')[0]}</td>
```

## 预防措施

1. **API 路径标准化**
   - 创建常量定义所有 API 路径
   - 统一使用 `/api/public/` 前缀

2. **接口测试**
   - 修改 Rust 代码前先用 curl 测试正确路径
   - 对比前后端 URL 是否一致

3. **日志调试**
   - 后端添加详细的请求/响应日志
   - 便于快速定位问题

## 教训

1. **路径一致性**
   - 测试代码和实际代码可能使用不同路径
   - 修改一处必须全局检查

2. **数据格式处理**
   - 后端返回原始数据，前端负责格式化展示
   - 日期、金额等需要统一处理

3. **调试技巧**
   - 对比成功和失败的请求日志是快速定位问题的方法
   - 添加 `println!` 日志对于异步 Rust 代码很有用

## 相关修改

- `src-tauri/src/aktools_client.rs` - 修正 API URL
- `src/App.tsx` - 日期格式化
