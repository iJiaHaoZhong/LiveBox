# 淘宝直播间集成说明

本项目已集成淘宝直播间弹幕抓取功能。

## 功能特性

- ✅ 支持淘宝直播间弹幕抓取
- ✅ 支持实时推送弹幕到配置的URL
- ✅ 自动识别淘宝直播间链接
- ✅ 与抖音直播间使用相同的UI界面

## 安装依赖

### Python 依赖

```bash
pip install -r requirements.txt
```

如果您只需要淘宝爬虫功能，可以单独安装：

```bash
pip install playwright loguru aiohttp
```

安装 Playwright 浏览器驱动：

```bash
playwright install chromium
```

## 使用方法

### 方法1：通过UI界面（推荐）

1. 启动LiveBox应用
2. 在输入框中输入淘宝直播间链接，格式如：
   ```
   https://tbzb.taobao.com/live?liveId=123456789
   ```
3. 点击"开始采集"按钮
4. 应用会自动识别淘宝链接并启动淘宝爬虫
5. 弹幕将显示在右侧消息列表中

### 方法2：直接运行Python脚本

如果您只想使用命令行运行淘宝爬虫：

```bash
# 基本用法
python3 taobao_crawler.py --room_id 直播间ID

# 指定输出文件
python3 taobao_crawler.py --room_id 直播间ID --output output.json

# 配置推送URL（实时推送弹幕）
python3 taobao_crawler.py --room_id 直播间ID --push_url http://localhost:5001/webhook
```

## 配置说明

### 推送URL格式

弹幕会以POST请求的形式推送到配置的URL，请求体格式：

```json
{
  "type": "chat",
  "platform": "taobao",
  "data": {
    "id": "消息ID",
    "name": "用户昵称",
    "msg": "弹幕内容"
  },
  "raw": {
    "timestamp": "2025-01-01T12:00:00",
    "user": "用户昵称",
    "content": "弹幕内容",
    "type": "chat"
  },
  "timestamp": 1704096000000,
  "room_id": "直播间ID"
}
```

### 接收推送示例

您可以使用项目中的 `example_receiver.py` 来接收推送的弹幕：

```bash
python3 example_receiver.py
```

默认监听端口：`http://localhost:5001/webhook`

## 技术架构

### 后端集成 (Rust)

- **文件**: `src-tauri/src/command/taobao.rs`
- **命令**:
  - `start_taobao_crawler`: 启动淘宝爬虫并监控输出
  - `stop_taobao_crawler`: 停止淘宝爬虫
  - `check_taobao_crawler_status`: 检查爬虫状态
- **事件**:
  - `taobao-log`: 实时推送爬虫日志到前端
    - `stdout`: 标准输出
    - `stderr`: 错误输出
    - `error`: 错误信息
    - `terminated`: 进程终止

### 前端集成 (Vue)

- **文件**: `src/App.vue`
- **功能**:
  - 自动识别淘宝链接并调用相应的后端命令
  - 监听 `taobao-log` 事件，实时显示爬虫状态
  - 在控制台显示详细日志
  - 根据日志内容显示用户友好的提示消息

### Python爬虫

- **文件**: `taobao_crawler.py`
- **技术**: Playwright + WebSocket + HTTP拦截
- **特性**:
  - 自动启动浏览器访问直播间
  - 拦截网络请求和WebSocket消息
  - 实时提取弹幕数据
  - 支持推送到外部URL

## 实时日志监控

淘宝爬虫的所有输出都会通过 Tauri 事件系统实时推送到前端：

```javascript
// 在前端监听日志
listen('taobao-log', (event) => {
    const { room_id, log_type, message } = event.payload
    console.log(`[淘宝爬虫 ${room_id}] [${log_type}] ${message}`)
})
```

日志类型：
- `stdout`: 标准输出（包括弹幕信息）
- `stderr`: 错误输出
- `error`: 错误信息
- `terminated`: 进程终止通知

## 常见问题

### 1. 启动失败提示找不到Python

**解决方法**：
- 确保已安装 Python 3.7+
- 运行 `python3 --version` 或 `python --version` 检查
- 确保 Python 在系统 PATH 中

### 2. 浏览器没有弹出

**可能原因**：
- Playwright 浏览器驱动未安装
- 进程启动失败

**解决方法**：
1. 查看应用控制台的日志输出
2. 运行 `playwright install chromium` 安装浏览器
3. 检查 Python 依赖是否完整：`pip install playwright loguru aiohttp`

### 3. 弹幕无法显示

**检查清单**：
- ✅ 推送URL是否正确配置
- ✅ 接收端服务是否正在运行（默认 http://localhost:5001/webhook）
- ✅ 查看控制台是否有 `taobao-log` 事件
- ✅ 查看 `logs/taobao_*.log` 日志文件
- ✅ 检查浏览器窗口是否显示"直播已结束"

### 4. 进程状态未知

现在通过 `taobao-log` 事件可以实时监控进程状态：
- 启动成功会看到 "✅ 淘宝爬虫进程已启动"
- 进程终止会收到 `terminated` 事件
- 所有 Python 输出都会实时推送

### 5. 需要登录

某些直播间可能需要登录才能查看弹幕。您可以：

1. 在弹出的浏览器窗口中手动登录淘宝
2. 或使用 Playwright 的 Cookie 持久化功能

## 与抖音爬虫的区别

| 特性 | 抖音直播间 | 淘宝直播间 |
|------|-----------|-----------|
| 连接方式 | 直接WebSocket | 浏览器自动化 |
| 登录要求 | Cookie可选 | 可能需要 |
| 资源占用 | 低 | 中等（需启动浏览器）|
| 稳定性 | 高 | 中等（依赖页面结构）|

## 更新日志

### v1.0.0 (2025-11-28)

- ✅ 初始集成淘宝直播间支持
- ✅ 支持弹幕实时推送
- ✅ UI自动识别淘宝链接
- ✅ 完整的日志记录

## 贡献

欢迎提交Issue和Pull Request来改进淘宝爬虫功能！

## 许可证

与主项目相同
