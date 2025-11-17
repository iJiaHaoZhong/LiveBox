# Python 实现快速开始指南

## 🚀 快速开始（5分钟上手）

### 第一步: 安装依赖

```bash
cd /path/to/LiveBox
pip install websockets requests protobuf
```

### 第二步: 编译 Protocol Buffers（可选但推荐）

```bash
# 安装 protoc 编译器
# macOS
brew install protobuf

# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# 编译 proto 文件
protoc --python_out=. src/proto/dy.proto
```

### 第三步: 运行监控脚本

```bash
python douyin_chat_monitor.py
```

输入直播间 URL，例如：
```
https://live.douyin.com/972176515698
```

就这么简单！

---

## 📝 详细说明

### 文件说明

| 文件 | 说明 |
|------|------|
| `douyin_chat_monitor.py` | 主程序 - 监控抖音直播间弹幕 |
| `requirements.txt` | Python 依赖包列表 |
| `generate_signature.js` | 签名生成辅助脚本（Node.js） |
| `PYTHON_IMPLEMENTATION.md` | 完整的使用文档 |
| `dy_pb2.py` | Protobuf 编译生成的文件（需自行编译） |

### 输出示例

```
============================================================
抖音直播间聊天弹幕监控
============================================================
正在获取直播间信息: https://live.douyin.com/972176515698
获取到 unique_id: 7347145653502019126
直播间正在直播中
直播间 ID: 7362491920259713818
主播昵称: 测试主播

正在连接 WebSocket...
URL: wss://webcast5-ws-web-lf.douyin.com/webcast/im/push/v2/...
✓ WebSocket 连接成功！
开始监听消息...

============================================================
[聊天] 张三: 主播你好！
[礼物] 李四 送出 玫瑰花 x5
[点赞] 王五 点赞了 (10)
[进入] 赵六 来了
[关注] 孙七 关注了主播
♥ 心跳
...
```

---

## ⚙️ 自定义配置

### 1. 只监听聊天消息

```python
from douyin_chat_monitor import DouyinLiveMonitor

monitor = DouyinLiveMonitor("https://live.douyin.com/your_room_id")

# 只设置聊天回调
monitor.set_chat_callback(lambda data: print(f"{data['name']}: {data['msg']}"))

monitor.start()
```

### 2. 保存到文件

```python
def save_to_file(data):
    with open('chat.txt', 'a', encoding='utf-8') as f:
        f.write(f"{data['name']}: {data['msg']}\n")

monitor.set_chat_callback(save_to_file)
```

### 3. 推送到服务器

```python
import requests

def push_to_server(data):
    requests.post('http://localhost:5000/webhook', json={
        'type': 'chat',
        'data': data
    })

monitor.set_chat_callback(push_to_server)
```

---

## 🔧 故障排除

### 问题 1: "ModuleNotFoundError: No module named 'dy_pb2'"

**原因**: 未编译 Protocol Buffers 文件

**解决**:
```bash
protoc --python_out=. src/proto/dy.proto
```

或者，脚本会自动降级到原始数据模式（不解析消息内容）。

### 问题 2: "WebSocket 连接失败"

**原因**: 签名验证失败

**解决方案 A** - 使用浏览器获取真实签名:
1. 打开浏览器开发者工具（F12）
2. 访问抖音直播间
3. 在 Network 标签找到 WebSocket 连接
4. 复制完整的 URL
5. 从 URL 中提取 `signature` 参数
6. 在代码中硬编码这个签名（有效期有限）

**解决方案 B** - 使用 PyExecJS 调用 JavaScript:
```bash
pip install PyExecJS
```

修改 `douyin_chat_monitor.py` 中的 `generate_signature` 方法。

### 问题 3: "未获取到 ttwid Cookie"

**原因**: 访问直播间页面时被反爬虫拦截

**解决**:
- 添加更真实的浏览器 Headers
- 使用代理 IP
- 降低请求频率
- 手动在浏览器中访问一次，复制 Cookie

### 问题 4: "IP 被限制"

**解决**:
- 等待一段时间（通常几小时）
- 更换网络环境
- 使用代理服务器

---

## 💡 使用技巧

### 技巧 1: 测试连接

使用一个正在直播的热门直播间测试：
```python
# 抖音官方账号或热门主播
monitor = DouyinLiveMonitor("https://live.douyin.com/official_account")
```

### 技巧 2: 调试模式

在代码中添加打印语句查看原始数据：
```python
def parse_message(self, data: bytes):
    print(f"收到 {len(data)} 字节")
    print(data[:50].hex())  # 打印前 50 字节
    # ...
```

### 技巧 3: 批量监控多个直播间

```python
import threading

def monitor_room(url):
    monitor = DouyinLiveMonitor(url)
    monitor.start()

rooms = [
    "https://live.douyin.com/room1",
    "https://live.douyin.com/room2",
    "https://live.douyin.com/room3",
]

for room in rooms:
    thread = threading.Thread(target=monitor_room, args=(room,))
    thread.start()
```

---

## 📊 性能优化

### 1. 使用异步处理

当前实现已经使用了 `asyncio`，但可以进一步优化：

```python
async def on_chat_async(data):
    """异步回调"""
    await save_to_database(data)

# 在 handle_message 中使用
asyncio.create_task(on_chat_async(data))
```

### 2. 消息队列缓冲

```python
from queue import Queue
import threading

message_queue = Queue()

def message_processor():
    """后台线程处理消息"""
    while True:
        data = message_queue.get()
        # 处理消息
        save_to_database(data)

# 启动处理线程
threading.Thread(target=message_processor, daemon=True).start()

# 在回调中只是入队
def on_chat(data):
    message_queue.put(data)
```

---

## 🎯 实际应用场景

### 场景 1: 弹幕词云分析

```python
from collections import Counter
import jieba

word_counter = Counter()

def on_chat(data):
    words = jieba.cut(data['msg'])
    word_counter.update(words)

# 定期输出热门词汇
import threading

def print_top_words():
    while True:
        time.sleep(60)  # 每分钟
        print("\n热门词汇 Top 10:")
        for word, count in word_counter.most_common(10):
            print(f"  {word}: {count}")

threading.Thread(target=print_top_words, daemon=True).start()
```

### 场景 2: 实时推送到前端

```python
from flask import Flask, jsonify
from flask_socketio import SocketIO, emit

app = Flask(__name__)
socketio = SocketIO(app, cors_allowed_origins="*")

def on_chat(data):
    """推送到所有连接的前端客户端"""
    socketio.emit('chat_message', data)

@app.route('/')
def index():
    return """
    <html>
    <script src="https://cdn.socket.io/4.5.4/socket.io.min.js"></script>
    <script>
        const socket = io();
        socket.on('chat_message', (data) => {
            console.log(data);
            // 显示在页面上
        });
    </script>
    </html>
    """

# 启动 Flask 服务器（后台线程）
threading.Thread(target=lambda: socketio.run(app, port=5000), daemon=True).start()

# 启动监控
monitor = DouyinLiveMonitor("https://live.douyin.com/...")
monitor.set_chat_callback(on_chat)
monitor.start()
```

### 场景 3: 关键词告警

```python
import smtplib
from email.mime.text import MIMEText

KEYWORDS = ['紧急', '求助', '问题']

def on_chat(data):
    msg = data['msg']

    # 检查关键词
    for keyword in KEYWORDS:
        if keyword in msg:
            send_alert(f"检测到关键词 '{keyword}': {data['name']}: {msg}")

def send_alert(message):
    """发送告警邮件/短信/推送通知"""
    print(f"⚠️ 告警: {message}")
    # 实现你的告警逻辑
```

---

## 🔒 安全建议

1. **不要公开分享签名**: 签名可能包含账号信息
2. **控制请求频率**: 避免被识别为爬虫
3. **遵守 robots.txt**: 尊重平台规则
4. **仅用于学习研究**: 不要用于商业用途
5. **保护隐私数据**: 不要泄露用户信息

---

## 📚 下一步学习

- 查看 `PYTHON_IMPLEMENTATION.md` 了解更多 API 和示例
- 查看 `HOW_TO_GET_CHAT_MESSAGES.md` 了解技术原理
- 查看 `MESSAGE_STRUCTURE.md` 了解消息格式

---

**创建时间**: 2025-11-17
**难度**: ⭐⭐⭐ (中等)
**预计时间**: 30 分钟 - 2 小时（取决于是否需要处理签名问题）
