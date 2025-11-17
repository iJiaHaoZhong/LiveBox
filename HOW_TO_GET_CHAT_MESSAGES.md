# LiveBox 聊天弹幕消息获取原理详解

## 📝 概述

LiveBox 通过 WebSocket 连接到抖音直播服务器，实时接收直播间的聊天弹幕消息。整个流程分为 4 个主要步骤：

1. **获取直播间信息** - 访问直播间网页，提取 room_id 等关键参数
2. **生成签名** - 计算 WebSocket 连接所需的签名
3. **建立 WebSocket 连接** - 连接到抖音的 WebSocket 服务器
4. **接收和解析消息** - 解码 Protocol Buffers 格式的消息

---

## 🔍 详细流程

### 第一步：获取直播间信息

#### 1.1 用户输入直播间地址

用户在界面输入抖音直播间 URL，例如：
```
https://live.douyin.com/972176515698
```

**代码位置**: `src/App.vue:208-268`

#### 1.2 调用 Rust 后端获取直播间信息

前端调用 Tauri 命令 `get_live_html`，由 Rust 后端处理：

```javascript
// src/App.vue:217
const roomJson: LiveInfoImp = await invoke('get_live_html', { url })
```

**Rust 处理逻辑** (`src-tauri/src/command/live.rs:15-25`):
```rust
#[tauri::command]
pub async fn get_live_html(url: &str) -> Result<LiveInfo, String> {
    let mut live_req = DouYinReq::new(url);
    let result = live_req.get_room_info().await;
    match result {
        Ok(info) => Ok(info),
        Err(_) => Err("This failed!".into()),
    }
}
```

#### 1.3 发送 HTTP 请求获取网页内容

**代码位置**: `src-tauri/src/command/runner.rs:23-83`

Rust 后端模拟浏览器访问直播间页面：

```rust
pub async fn get_room_info(&mut self) -> Result<LiveInfo, Box<dyn std::error::Error>> {
    // 1. 构建请求头，模拟真实浏览器
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) ...".parse()?);
    headers.insert("accept", "text/html,application/xhtml+xml,...".parse()?);
    // ... 更多请求头

    // 2. 发送请求
    let request = self.request.get(self.room_url.clone()).headers(headers);
    let response = request.send().await?;

    // 3. 提取 Cookie 中的 ttwid（非常重要！）
    let cookies = response.cookies();
    let mut ttwid = String::new();
    for c in cookies {
        if c.name() == "ttwid" {
            ttwid = c.value().to_string();
        }
    }

    // 4. 获取 HTML 内容
    let body = response.text().await?;

    // 5. 使用正则表达式提取关键信息
    if body.contains(r#"status\":4"#) {
        // 直播已结束
        re = Regex::new(r#"anchor\\":(.*?),\\"open_id_str"#).unwrap();
    } else {
        // 直播进行中
        re = Regex::new(r#"roomInfo\\":\{\\"room\\":(.*?),\\"toolbar_data"#).unwrap();
        let unique_re = Regex::new(r#"user_unique_id\\":\\"(.*?)\\"}"#).unwrap();
        unique_id = unique_re.captures(&body).unwrap().get(1).unwrap().as_str();
    }

    let main_info = re.captures(&body).unwrap().get(1).unwrap().as_str();
    let room_info = String::from(main_info) + "}";
    self.room_info = room_info.replace(r#"\""#, r#"""#);

    // 6. 返回关键信息
    Ok(LiveInfo {
        room_info: self.room_info.clone(),  // 直播间详细信息（JSON）
        ttwid,                               // Cookie 中的 ttwid
        unique_id: String::from(unique_id),  // 用户唯一ID
    })
}
```

#### 1.4 提取关键参数

从 HTML 页面中提取的关键信息：

| 参数 | 说明 | 来源 | 用途 |
|------|------|------|------|
| `room_id` | 直播间ID | HTML 中的 roomInfo.id_str | WebSocket URL 参数 |
| `ttwid` | 抖音跟踪ID | HTTP 响应的 Cookie | WebSocket 请求头 |
| `unique_id` | 用户唯一ID | HTML 中的 user_unique_id | 生成签名 |

**代码位置**: `src/App.vue:220-245`

```javascript
const roomInfo = JSON.parse(roomJson.room_info)
liveInfo.value = {
    uid: roomInfo.owner.id_str,
    status: roomInfo.status,
    title: roomInfo.title,
    name: roomInfo.owner.nickname,
    roomId: roomInfo.id_str,  // 这个是关键！
    avatar: roomInfo.owner.avatar_thumb.url_list[0],
    // ...
}

// 准备创建 WebSocket 连接
creatSokcet(roomInfo.id_str, roomJson.unique_id, roomJson.ttwid)
```

---

### 第二步：生成签名

#### 2.1 为什么需要签名？

抖音 WebSocket 服务器需要验证请求的合法性，防止恶意爬虫。签名参数 `signature` 是必需的。

#### 2.2 签名生成过程

**代码位置**: `src/App.vue:287` 和 `src/assets/static/vFun.js:166-193`

```javascript
// 1. 调用签名函数
let sign = window.creatSignature(roomId, uniqueId)

// 2. 签名生成算法 (vFun.js:166-193)
window.creatSignature = (roomId, uniqueId) => {
    // 构建签名字符串
    const o = `,live_id=1,aid=6383,version_code=180800,webcast_sdk_version=1.0.14-beta.0,room_id=${roomId},sub_room_id=,sub_channel_id=,did_rule=3,user_unique_id=${uniqueId},device_platform=web,device_type=,ac=,identity=audience`

    // 去掉开头的逗号
    const substr = o.substring(1)

    // MD5 哈希处理
    const sResult = sFunc(substr)  // stringToBytes
    const r = wordsToBytes(sResult)
    const bytesRes = bytesToHex(r)

    // 使用抖音的加密算法生成最终签名
    const frontierSignRes = window.byted_acrawler.frontierSign({
        'X-MS-STUB': bytesRes,
    })

    // 返回 X-Bogus 签名
    return frontierSignRes['X-Bogus']
}
```

**关键依赖**:
- `window.byted_acrawler` - 抖音官方的 JavaScript 加密库（在 `src/assets/static/model.js` 中）
- 签名算法结合了 MD5 哈希和抖音专有的加密方法

---

### 第三步：建立 WebSocket 连接

#### 3.1 构建 WebSocket URL

**代码位置**: `src/App.vue:285-310`

```javascript
const creatSokcet = async (roomId: string, uniqueId: string, ttwid: string) => {
    // 1. 生成签名
    let sign = window.creatSignature(roomId, uniqueId)

    // 2. 构建完整的 WebSocket URL
    let socketUrl = `wss://webcast5-ws-web-lf.douyin.com/webcast/im/push/v2/?room_id=${roomId}&compress=gzip&version_code=180800&webcast_sdk_version=1.0.14-beta.0&live_id=1&did_rule=3&user_unique_id=${uniqueId}&identity=audience&signature=${sign}&aid=6383&device_platform=web&browser_language=zh-CN&browser_platform=Win32&browser_name=Mozilla&browser_version=5.0+...`

    // 3. 配置 WebSocket 选项
    const options: ConnectionConfig = {
        writeBufferSize: 20000,
        headers: {
            cookie: 'ttwid=' + ttwid,  // 必须携带 ttwid
            'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) ...',
        },
    }

    // 4. 创建心跳消息
    const pingMsg = douyin.PushFrame.encode({ payloadType: 'hb' }).finish()

    // 5. 建立连接
    socketClient = new SocketCli(socketUrl, options, onMessage, pingMsg)
}
```

#### 3.2 WebSocket URL 参数说明

| 参数 | 值 | 说明 |
|------|-----|------|
| `room_id` | 直播间ID | 从第一步获取 |
| `compress` | gzip | 消息使用 gzip 压缩 |
| `version_code` | 180800 | 客户端版本号 |
| `webcast_sdk_version` | 1.0.14-beta.0 | SDK 版本 |
| `user_unique_id` | 用户唯一ID | 从第一步获取 |
| `identity` | audience | 身份：观众 |
| `signature` | 签名字符串 | 从第二步生成 |
| `aid` | 6383 | 应用ID |
| `device_platform` | web | 平台：网页端 |

#### 3.3 WebSocket 连接封装

**代码位置**: `src/utils/RustSocket.ts`

```typescript
class SocketCli {
    constructor(url, options, onMessageCallback, pingMsg) {
        this.url = url
        this.options = options
        this.onMessage = onMessageCallback
        this.pingMsg = pingMsg

        this.connect()
    }

    async connect() {
        // 使用 Tauri 的 WebSocket API 建立连接
        this.socket = await WebSocket.connect(this.url, this.options)

        // 监听消息
        this.socket.addListener((msg) => {
            this.onMessage(msg)
        })

        // 启动心跳
        this.startHeartbeat()
    }

    startHeartbeat() {
        // 每 10 秒发送一次心跳
        this.heartbeatTimer = setInterval(() => {
            this.send(this.pingMsg)
        }, 10000)
    }
}
```

---

### 第四步：接收和解析消息

#### 4.1 消息接收回调

**代码位置**: `src/App.vue:388-416`

```javascript
const onMessage = (msg: any) => {
    // 1. 解码 PushFrame（外层封装）
    const decodeMsg = douyin.PushFrame.decode(msg.data)

    // 2. gzip 解压缩 payload
    const gzipData = pako.inflate(decodeMsg.payload)

    // 3. 解码 Response（消息列表）
    const response = douyin.Response.decode(gzipData)

    // 4. 如果需要 ACK，发送确认
    if (response.needAck) {
        const ack = douyin.PushFrame.encode({
            payloadType: 'ack',
            logId: decodeMsg.logId,
        }).finish()
        socketClient?.send(ack)
    }

    // 5. 处理消息列表
    handleMessage(response.messagesList)
}
```

#### 4.2 消息解析流程

```
原始二进制数据（WebSocket 接收）
    ↓
PushFrame.decode() ← 解码外层封装
    ↓
PushFrame.payload（gzip 压缩的数据）
    ↓
pako.inflate() ← gzip 解压
    ↓
Response（包含多条消息）
    ↓
Response.messagesList（消息数组）
    ↓
遍历每条消息，根据 method 类型分发
    ↓
ChatMessage.decode() ← 解码聊天消息
    ↓
提取用户昵称和聊天内容
```

#### 4.3 消息分发和处理

**代码位置**: `src/App.vue:419-475`

```javascript
const handleMessage = (messageList: douyin.Message) => {
    messageList.forEach((msg) => {
        // 根据消息类型分发
        switch (msg.method) {
            case 'WebcastChatMessage':
                // 聊天弹幕消息
                decodeChat(msg.payload)
                break

            case 'WebcastGiftMessage':
                // 礼物消息
                decodeGift(msg.payload)
                break

            case 'WebcastLikeMessage':
                // 点赞消息
                likeLive(msg.payload)
                break

            case 'WebcastMemberMessage':
                // 进入直播间消息
                enterLive(msg.payload)
                break

            case 'WebcastSocialMessage':
                // 关注消息
                followLive(msg.payload)
                break

            default:
                console.log('待解析方法' + msg.method)
                break
        }
    })
}
```

#### 4.4 聊天消息解析（重点！）

**代码位置**: `src/App.vue:477-489`

```javascript
const decodeChat = (data) => {
    // 1. 使用 Protocol Buffers 解码
    const chatMsg = douyin.ChatMessage.decode(data)

    // 2. 提取关键字段
    const { common, user, content } = chatMsg

    // 3. 组装消息对象
    const message = {
        id: common.msgId,       // 消息ID
        name: user.nickName,    // 用户昵称
        msg: content,           // 聊天内容
    }

    // 4. 如果用户选中了聊天消息类型，添加到显示列表
    checkList.value.includes('chat') && messageList.value.push(message)
}
```

**ChatMessage 结构** (定义在 `src/proto/dy.proto:32-53`):

```protobuf
message ChatMessage {
  Common common = 1;    // 公共字段（msgId, roomId, createTime等）
  User user = 2;        // 用户信息（id, nickName, avatar等）
  string content = 3;   // 聊天内容
}
```

---

## 💡 关键技术点总结

### 1. Protocol Buffers 编解码

LiveBox 使用 Google 的 Protocol Buffers 协议解析消息：

- **定义文件**: `src/proto/dy.proto` - 定义所有消息结构
- **编译后**: `src/proto/dy.js` - JavaScript 版本的编解码器
- **库**: `protobufjs` - JavaScript 实现

```javascript
// 编码（发送）
const frame = douyin.PushFrame.encode({ payloadType: 'hb' }).finish()
socketClient.send(frame)

// 解码（接收）
const message = douyin.ChatMessage.decode(binaryData)
```

### 2. gzip 压缩

抖音为了节省带宽，所有 WebSocket 消息都经过 gzip 压缩：

```javascript
import pako from 'pako'

// 解压缩
const decompressed = pako.inflate(compressedData)
```

### 3. 签名算法

关键点：
- 结合 `room_id` 和 `user_unique_id`
- 使用 MD5 哈希
- 调用抖音的 `byted_acrawler.frontierSign()` 生成最终签名
- 签名时效性：需要及时生成，过期会连接失败

### 4. Cookie 的重要性

`ttwid` Cookie 必须携带：
- 从访问直播间页面的响应中获取
- 在 WebSocket 连接时通过 `headers.cookie` 传递
- 用于身份识别和防爬

### 5. 心跳保持连接

```javascript
// 每 10 秒发送一次心跳
const pingMsg = douyin.PushFrame.encode({ payloadType: 'hb' }).finish()
setInterval(() => {
    socketClient.send(pingMsg)
}, 10000)
```

---

## 📊 完整数据流图

```
┌─────────────────┐
│ 用户输入直播间URL │
└────────┬────────┘
         ↓
┌────────────────────────┐
│ Rust: 访问直播间网页     │
│ - 模拟浏览器请求头       │
│ - 获取 ttwid Cookie    │
│ - 正则提取 roomInfo    │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ 提取关键参数             │
│ - room_id (直播间ID)   │
│ - unique_id (用户ID)   │
│ - ttwid (Cookie)       │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ JS: 生成签名            │
│ - MD5 哈希处理          │
│ - byted_acrawler 加密  │
│ - 得到 signature       │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ 建立 WebSocket 连接     │
│ URL: wss://webcast5... │
│ Params: room_id, sign  │
│ Headers: ttwid cookie  │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ 接收二进制消息           │
│ ↓ PushFrame.decode()   │
│ ↓ pako.inflate()       │
│ ↓ Response.decode()    │
│ ↓ messagesList         │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ 根据 method 分发消息    │
│ - WebcastChatMessage   │
│ - WebcastGiftMessage   │
│ - WebcastLikeMessage   │
│ - ...                  │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ ChatMessage.decode()   │
│ 提取：                  │
│ - user.nickName        │
│ - content              │
│ - common.msgId         │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ 显示在聊天列表           │
│ 或推送到配置的 URL      │
└────────────────────────┘
```

---

## 🎯 核心文件索引

| 文件 | 说明 | 关键代码行 |
|------|------|-----------|
| `src/App.vue` | 主界面，WebSocket 连接和消息处理 | 208-559 |
| `src-tauri/src/command/runner.rs` | Rust 后端，获取直播间信息 | 23-83 |
| `src-tauri/src/command/live.rs` | Tauri 命令定义 | 15-25 |
| `src/assets/static/vFun.js` | 签名生成算法 | 166-193 |
| `src/assets/static/model.js` | 抖音加密库 byted_acrawler | 全文 |
| `src/proto/dy.proto` | Protocol Buffers 消息定义 | 32-53 (ChatMessage) |
| `src/proto/dy.js` | 编译后的 Protobuf 编解码器 | 自动生成 |
| `src/utils/RustSocket.ts` | WebSocket 封装（心跳、重连） | 全文 |

---

## 🔐 安全和合规说明

1. **仅供学习研究**：此项目用于学习 WebSocket 和 Protocol Buffers 技术
2. **Cookie 获取**：通过正常浏览器访问获取，非盗取
3. **不涉及账号登录**：仅作为游客身份观看直播
4. **不破坏平台功能**：只接收公开的直播消息，不发送任何消息
5. **请遵守抖音服务条款**

---

## 📚 相关技术文档

- [Protocol Buffers 官方文档](https://protobuf.dev/)
- [WebSocket API (MDN)](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [Pako gzip 库](https://github.com/nodeca/pako)
- [Tauri WebSocket Plugin](https://tauri.app/v1/api/js/websocket/)

---

**文档更新时间**: 2025-11-17
**适用版本**: LiveBox 当前版本
**维护者**: Claude
