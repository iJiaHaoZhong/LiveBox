# Python 实现抖音直播间弹幕监控

## 📦 安装依赖

```bash
pip install -r requirements.txt
```

或手动安装：
```bash
pip install websockets requests protobuf
```

## 🔧 编译 Protocol Buffers

**重要**: 要使用完整的消息解析功能，需要先将 `.proto` 文件编译为 Python 代码。

### 1. 安装 protoc 编译器

**macOS**:
```bash
brew install protobuf
```

**Ubuntu/Debian**:
```bash
sudo apt-get install protobuf-compiler
```

**Windows**:
从 [GitHub Releases](https://github.com/protocolbuffers/protobuf/releases) 下载预编译版本

### 2. 编译 proto 文件

```bash
# 在项目根目录执行
protoc --python_out=. src/proto/dy.proto
```

这会生成 `dy_pb2.py` 文件，包含所有消息类型的 Python 定义。

### 3. 验证编译

```bash
python -c "from dy_pb2 import ChatMessage; print('✓ Protobuf 编译成功')"
```

## 🚀 使用方法

### 基础使用

```python
from douyin_chat_monitor import DouyinLiveMonitor

# 创建监控器
monitor = DouyinLiveMonitor("https://live.douyin.com/your_room_id")

# 启动监控
monitor.start()
```

### 运行脚本

```bash
python douyin_chat_monitor.py
```

然后输入直播间 URL。

### 带回调函数的使用

```python
from douyin_chat_monitor import DouyinLiveMonitor

def on_chat(data):
    """处理聊天消息"""
    print(f"收到聊天: {data['name']}: {data['msg']}")
    # 保存到数据库、推送到其他服务等

def on_gift(data):
    """处理礼物消息"""
    print(f"收到礼物: {data['name']} 送了 {data['gift_name']} x{data['gift_count']}")

# 创建监控器
monitor = DouyinLiveMonitor("https://live.douyin.com/972176515698")

# 设置回调
monitor.set_chat_callback(on_chat)
monitor.set_gift_callback(on_gift)

# 启动
monitor.start()
```

## 📝 API 说明

### DouyinLiveMonitor 类

#### 初始化

```python
monitor = DouyinLiveMonitor(live_url: str)
```

- `live_url`: 抖音直播间 URL

#### 方法

| 方法 | 说明 |
|------|------|
| `set_chat_callback(callback)` | 设置聊天消息回调 |
| `set_gift_callback(callback)` | 设置礼物消息回调 |
| `set_like_callback(callback)` | 设置点赞消息回调 |
| `set_member_callback(callback)` | 设置进入房间消息回调 |
| `set_follow_callback(callback)` | 设置关注消息回调 |
| `start()` | 启动监控（阻塞） |

#### 回调函数数据格式

**聊天消息**:
```python
{
    'id': '消息ID',
    'name': '用户昵称',
    'msg': '聊天内容'
}
```

**礼物消息**:
```python
{
    'id': '消息ID',
    'name': '用户昵称',
    'gift_name': '礼物名称',
    'gift_count': 123,  # 数量
    'diamond_count': 10  # 单价（抖币）
}
```

**点赞消息**:
```python
{
    'id': '消息ID',
    'name': '用户昵称',
    'count': 10,    # 本次点赞数
    'total': 12345  # 累计点赞数
}
```

## ⚠️ 已知限制

### 1. 签名算法

当前实现使用的是**简化版签名**，可能无法成功连接到 WebSocket。

完整的签名算法依赖于抖音的 JavaScript 加密库 `byted_acrawler`，实现方式：

**方案 A: 使用 PyExecJS（推荐）**

```bash
pip install PyExecJS
```

```python
import execjs

# 读取 JavaScript 签名代码
with open('src/assets/static/vFun.js', 'r', encoding='utf-8') as f:
    js_code = f.read()

with open('src/assets/static/model.js', 'r', encoding='utf-8') as f:
    js_code += f.read()

# 编译 JavaScript 上下文
ctx = execjs.compile(js_code)

# 调用签名函数
signature = ctx.call('creatSignature', room_id, unique_id)
```

**方案 B: 使用 Node.js 子进程**

```python
import subprocess
import json

def generate_signature(room_id, unique_id):
    result = subprocess.run(
        ['node', 'generate_sign.js', room_id, unique_id],
        capture_output=True,
        text=True
    )
    return result.stdout.strip()
```

**方案 C: 抓包获取现成签名**

从浏览器开发者工具中复制 WebSocket URL 里的 `signature` 参数。

### 2. Protobuf 可选

如果不编译 `.proto` 文件，脚本仍可运行，但无法解析消息内容，只能看到原始二进制数据。

### 3. 反爬虫机制

抖音有反爬虫措施，频繁请求可能导致：
- IP 被限制
- 需要验证码
- 签名失效

建议：
- 控制请求频率
- 使用代理 IP
- 模拟真实用户行为

## 🔍 调试技巧

### 1. 查看原始数据

```python
def parse_message(self, data: bytes):
    print(f"收到 {len(data)} 字节数据")
    print(f"前 100 字节: {data[:100].hex()}")
    # ... 继续解析
```

### 2. 保存消息到文件

```python
def on_chat(data):
    with open('chat_log.txt', 'a', encoding='utf-8') as f:
        f.write(f"{data['name']}: {data['msg']}\n")
```

### 3. 使用 Wireshark 抓包

对比 LiveBox 和 Python 实现的数据包，检查差异。

## 📊 完整示例：保存聊天记录到数据库

```python
import sqlite3
from datetime import datetime
from douyin_chat_monitor import DouyinLiveMonitor

# 创建数据库
conn = sqlite3.connect('douyin_chat.db')
cursor = conn.cursor()

cursor.execute('''
    CREATE TABLE IF NOT EXISTS chat_messages (
        id TEXT PRIMARY KEY,
        room_id TEXT,
        user_name TEXT,
        message TEXT,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
    )
''')
conn.commit()

def on_chat_message(data):
    """保存聊天消息到数据库"""
    cursor.execute(
        'INSERT OR IGNORE INTO chat_messages (id, user_name, message) VALUES (?, ?, ?)',
        (data['id'], data['name'], data['msg'])
    )
    conn.commit()
    print(f"[保存] {data['name']}: {data['msg']}")

# 启动监控
monitor = DouyinLiveMonitor("https://live.douyin.com/972176515698")
monitor.set_chat_callback(on_chat_message)

try:
    monitor.start()
finally:
    conn.close()
```

## 📊 完整示例：推送到 HTTP 服务器

```python
import requests
from douyin_chat_monitor import DouyinLiveMonitor

def on_chat_message(data):
    """推送到 HTTP 服务器"""
    try:
        requests.post(
            'http://localhost:5000/webhook',
            json={
                'type': 'chat',
                'data': data
            },
            timeout=5
        )
        print(f"[推送成功] {data['name']}: {data['msg']}")
    except Exception as e:
        print(f"[推送失败] {e}")

monitor = DouyinLiveMonitor("https://live.douyin.com/972176515698")
monitor.set_chat_callback(on_chat_message)
monitor.start()
```

## 🔗 与 LiveBox 的对比

| 功能 | LiveBox (Tauri + Rust) | Python 实现 |
|------|----------------------|-----------|
| 获取直播间信息 | ✅ Rust HTTP 客户端 | ✅ requests |
| 签名生成 | ✅ JavaScript (完整) | ⚠️ 简化版 |
| WebSocket 连接 | ✅ Tauri Plugin | ✅ websockets |
| Protobuf 解析 | ✅ protobuf.js | ✅ protobuf (需编译) |
| gzip 解压 | ✅ pako | ✅ gzip 标准库 |
| GUI 界面 | ✅ Vue.js | ❌ 命令行 |
| 跨平台打包 | ✅ Tauri | ❌ 需要 PyInstaller |

## 🛠️ 进阶改进

### 1. 添加重连机制

```python
async def connect_with_retry(self, max_retries=5):
    for attempt in range(max_retries):
        try:
            await self.connect()
            break
        except Exception as e:
            print(f"连接失败 (尝试 {attempt + 1}/{max_retries}): {e}")
            if attempt < max_retries - 1:
                await asyncio.sleep(5)
```

### 2. 添加消息过滤

```python
class DouyinLiveMonitor:
    def __init__(self, live_url, message_types=['chat']):
        self.message_types = message_types
        # ...

    def handle_message(self, msg):
        method = msg.method

        if 'chat' not in self.message_types and method == 'WebcastChatMessage':
            return  # 跳过聊天消息

        # ... 继续处理
```

### 3. 添加日志记录

```python
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    handlers=[
        logging.FileHandler('douyin_monitor.log'),
        logging.StreamHandler()
    ]
)
```

## 📚 参考资料

- LiveBox 原理文档: `HOW_TO_GET_CHAT_MESSAGES.md`
- 消息结构文档: `MESSAGE_STRUCTURE.md`
- Protocol Buffers 官方文档: https://protobuf.dev/
- Python websockets 文档: https://websockets.readthedocs.io/

## ⚖️ 免责声明

本实现仅供学习和研究使用，请遵守抖音平台的服务条款。不得用于：
- 商业用途
- 恶意爬虫
- 干扰平台正常运行
- 其他违法违规行为

---

**创建时间**: 2025-11-17
**基于**: LiveBox 项目原理
**作者**: Claude
