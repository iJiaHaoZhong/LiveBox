# LiveBox 推送消息结构文档

## 重要说明 ⚠️

**当前状态**: LiveBox 项目中已经有推送地址配置的 UI 界面，但**实际的 HTTP 推送功能尚未实现**。

- ✅ 配置界面完整 (`src/App.vue:74-120`)
- ✅ 消息类型选择器完整
- ❌ **HTTP POST 推送逻辑未实现**

如果需要实现推送功能，需要在消息处理函数中添加 HTTP POST 请求代码。

---

## 一、推送地址配置

### 配置位置
- **文件**: `src/App.vue`
- **UI 配置**: 第 74-120 行
- **变量定义**: 第 170 行

```vue
<!-- 设置按钮 -->
<el-icon :size="20" class="pushUrl" @click="dialogVisible = true">
    <Setting />
</el-icon>

<!-- 配置对话框 -->
<el-dialog v-model="dialogVisible" title="设置推送地址">
    <el-input v-model="pushUrl" placeholder="请输入推送地址" />
    <!-- 消息类型选择 -->
    <el-checkbox-group v-model="checkList">
        <el-checkbox label="聊天" value="chat" />
        <el-checkbox label="礼物" value="gift" />
        <el-checkbox label="点赞" value="like" />
        <el-checkbox label="关注" value="follow" />
        <el-checkbox label="进来" value="comein" />
    </el-checkbox-group>
</el-dialog>
```

### 推送方式
- **HTTP 方法**: POST
- **Content-Type**: application/json
- **消息格式**: JSON

---

## 二、消息类型一览

| 消息类型 | 标识 | WebSocket Method | Proto 定义 | 处理函数位置 | 默认启用 |
|---------|------|------------------|-----------|-------------|---------|
| 聊天弹幕 | `chat` | `WebcastChatMessage` | `ChatMessage` | `App.vue:477` | ✅ |
| 礼物 | `gift` | `WebcastGiftMessage` | `GiftMessage` | `App.vue:491` | ✅ |
| 点赞 | `like` | `WebcastLikeMessage` | `LikeMessage` | `App.vue:519` | ✅ |
| 关注 | `follow` | `WebcastSocialMessage` | `SocialMessage` | `App.vue:536` | ❌ |
| 进入直播间 | `comein` | `WebcastMemberMessage` | `MemberMessage` | `App.vue:506` | ❌ |

---

## 三、消息数据结构详解

### 通用字段说明

所有消息都包含以下结构：
```json
{
  "type": "消息类型标识 (chat/gift/like/follow/comein)",
  "data": {
    "id": "消息唯一ID (字符串)",
    "name": "用户昵称 (字符串)",
    "msg": "消息内容描述 (字符串)"
  },
  "raw": {
    "// 原始的 Protocol Buffers 解码数据"
  }
}
```

### 1. 聊天消息 (chat)

#### 简化数据
```json
{
  "type": "chat",
  "data": {
    "id": "7123456789012345678",
    "name": "张三",
    "msg": "主播你好！"
  }
}
```

#### 完整原始数据 (raw)
```json
{
  "type": "chat",
  "data": { ... },
  "raw": {
    "common": {
      "method": "WebcastChatMessage",
      "msgId": 7123456789012345678,
      "roomId": 7234567890123456789,
      "createTime": 1700000000000,
      "isShowMsg": true
    },
    "user": {
      "id": 123456789,
      "shortId": 0,
      "nickName": "张三",
      "gender": 1,
      "level": 15,
      "avatarThumb": {
        "urlListList": [
          "https://p3.douyinpic.com/aweme/100x100/xxxxx.jpeg"
        ]
      }
    },
    "content": "主播你好！"
  }
}
```

#### Proto 定义
```protobuf
message ChatMessage {
  Common common = 1;
  User user = 2;
  string content = 3;
}
```

**关键字段**:
- `common.msgId`: 消息唯一ID
- `user.nickName`: 用户昵称
- `user.avatarThumb.urlListList[0]`: 用户头像URL
- `content`: 聊天内容

---

### 2. 礼物消息 (gift)

#### 简化数据
```json
{
  "type": "gift",
  "data": {
    "id": "7123456789012345679",
    "name": "李四",
    "msg": "送出玫瑰花 x5个"
  }
}
```

#### 完整原始数据 (raw)
```json
{
  "type": "gift",
  "data": { ... },
  "raw": {
    "common": {
      "msgId": 7123456789012345679,
      "roomId": 7234567890123456789,
      "createTime": 1700000000000
    },
    "user": {
      "id": 987654321,
      "nickName": "李四",
      "avatarThumb": {
        "urlListList": ["https://p3.douyinpic.com/..."]
      }
    },
    "gift": {
      "id": 6221,
      "name": "玫瑰花",
      "diamondCount": 1,
      "type": 1,
      "icon": {
        "urlListList": ["https://p3.douyinpic.com/..."]
      },
      "describe": "送你一朵玫瑰花"
    },
    "repeatCount": 5,
    "comboCount": 1,
    "totalCount": 5
  }
}
```

#### Proto 定义
```protobuf
message GiftMessage {
  Common common = 1;
  User user = 7;
  GiftStruct gift = 15;
  uint64 repeatCount = 5;
  uint64 comboCount = 6;
  uint64 totalCount = 29;
}

message GiftStruct {
  uint64 id = 5;
  string name = 16;
  uint32 diamondCount = 12;  // 单个礼物价值（抖币）
  Image icon = 21;
  string describe = 2;
}
```

**关键字段**:
- `gift.name`: 礼物名称
- `gift.diamondCount`: 单个礼物价值（抖币，1抖币≈0.1元）
- `repeatCount`: 连击次数
- **总价值计算**: `diamondCount × repeatCount`

---

### 3. 点赞消息 (like)

#### 简化数据
```json
{
  "type": "like",
  "data": {
    "id": "7123456789012345680",
    "name": "王五",
    "msg": "为主播点赞了"
  }
}
```

#### 完整原始数据 (raw)
```json
{
  "type": "like",
  "data": { ... },
  "raw": {
    "common": {
      "msgId": 7123456789012345680,
      "roomId": 7234567890123456789,
      "createTime": 1700000000000
    },
    "user": {
      "id": 456789123,
      "nickName": "王五"
    },
    "count": 10,
    "total": 123456
  }
}
```

#### Proto 定义
```protobuf
message LikeMessage {
  Common common = 1;
  User user = 5;
  uint64 count = 2;   // 本次点赞数
  uint64 total = 3;   // 累计总点赞数
}
```

**关键字段**:
- `count`: 本次点赞数量
- `total`: 直播间累计总点赞数

---

### 4. 关注消息 (follow)

#### 简化数据
```json
{
  "type": "follow",
  "data": {
    "id": "7123456789012345681",
    "name": "赵六",
    "msg": "关注了主播"
  }
}
```

#### 完整原始数据 (raw)
```json
{
  "type": "follow",
  "data": { ... },
  "raw": {
    "common": {
      "msgId": 7123456789012345681,
      "roomId": 7234567890123456789,
      "createTime": 1700000000000
    },
    "user": {
      "id": 789123456,
      "nickName": "赵六"
    },
    "followCount": 12345,
    "action": 1,
    "shareType": 0
  }
}
```

#### Proto 定义
```protobuf
message SocialMessage {
  Common common = 1;
  User user = 2;
  uint64 followCount = 6;  // 主播粉丝总数
  uint64 action = 4;
}
```

**关键字段**:
- `followCount`: 主播当前粉丝总数
- `action`: 动作类型（1=关注）

---

### 5. 进入直播间消息 (comein)

#### 简化数据
```json
{
  "type": "comein",
  "data": {
    "id": "7123456789012345682",
    "name": "孙七",
    "msg": "来了"
  }
}
```

#### 完整原始数据 (raw)
```json
{
  "type": "comein",
  "data": { ... },
  "raw": {
    "common": {
      "msgId": 7123456789012345682,
      "roomId": 7234567890123456789,
      "createTime": 1700000000000
    },
    "user": {
      "id": 321654987,
      "nickName": "孙七",
      "avatarThumb": {
        "urlListList": ["https://p3.douyinpic.com/..."]
      },
      "level": 8
    },
    "memberCount": 567,
    "enterType": 1
  }
}
```

#### Proto 定义
```protobuf
message MemberMessage {
  Common common = 1;
  User user = 2;
  uint64 memberCount = 3;  // 直播间成员总数
  uint64 enterType = 9;
}
```

**关键字段**:
- `memberCount`: 直播间当前成员数
- `enterType`: 进入类型

---

## 四、User 对象详解

所有消息中的 `user` 字段结构：

```protobuf
message User {
  uint64 id = 1;              // 用户ID
  uint64 shortId = 2;         // 短ID
  string nickName = 3;        // 昵称
  uint32 gender = 4;          // 性别 (0=未知, 1=男, 2=女)
  uint32 Level = 6;           // 等级
  Image AvatarThumb = 9;      // 头像缩略图
  Image AvatarMedium = 10;    // 中等尺寸头像
  Image AvatarLarge = 11;     // 大尺寸头像
  string displayId = 38;      // 显示ID
  string secUid = 46;         // 安全UID
  uint64 fanTicketCount = 1022; // 粉丝票数
}
```

---

## 五、Common 对象详解

所有消息中的 `common` 字段结构：

```protobuf
message Common {
  string method = 1;          // 消息方法名
  uint64 msgId = 2;           // 消息唯一ID
  uint64 roomId = 3;          // 直播间ID
  uint64 createTime = 4;      // 创建时间戳（毫秒）
  bool isShowMsg = 6;         // 是否显示消息
  string describe = 7;        // 描述
}
```

---

## 六、消息处理流程

### 当前实现 (`src/App.vue:388-559`)

```
WebSocket 接收
    ↓
PushFrame.decode() - 解码推送帧
    ↓
pako.inflate() - gzip 解压
    ↓
Response.decode() - 解码响应
    ↓
遍历 messagesList
    ↓
根据 method 分发到不同处理函数
    ↓
解码具体消息类型
    ↓
添加到 messageList 显示在 UI
```

### 如需实现 HTTP 推送，需要添加

在每个消息处理函数（`decodeChat`, `decodeGift` 等）中添加：

```javascript
// 示例：在 decodeChat 函数中添加
const decodeChat = async (data) => {
    const chatMsg = douyin.ChatMessage.decode(data)
    const { common, user, content } = chatMsg

    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: content,
    }

    // 原有逻辑
    checkList.value.includes('chat') && messageList.value.push(message)

    // 新增：HTTP 推送逻辑
    if (pushUrl.value && checkList.value.includes('chat')) {
        try {
            await fetch(pushUrl.value, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    type: 'chat',
                    data: message,
                    raw: chatMsg
                })
            })
        } catch (error) {
            console.error('推送失败:', error)
        }
    }
}
```

---

## 七、接收端程序示例

### Python Flask 示例

运行方法：
```bash
pip install flask
python example_receiver.py
```

配置地址：`http://localhost:5000/webhook`

### Node.js Express 示例

运行方法：
```bash
npm install express
node example_receiver_nodejs.js
```

配置地址：`http://localhost:3000/webhook`

---

## 八、测试推送功能

### 1. 使用 curl 测试

```bash
# 测试聊天消息
curl -X POST http://localhost:5000/webhook \
  -H "Content-Type: application/json" \
  -d '{
    "type": "chat",
    "data": {
      "id": "123456789",
      "name": "测试用户",
      "msg": "这是一条测试消息"
    }
  }'

# 测试礼物消息
curl -X POST http://localhost:5000/webhook \
  -H "Content-Type: application/json" \
  -d '{
    "type": "gift",
    "data": {
      "id": "123456790",
      "name": "测试用户",
      "msg": "送出玫瑰花 x5个"
    },
    "raw": {
      "gift": {
        "name": "玫瑰花",
        "diamondCount": 1
      },
      "repeatCount": 5
    }
  }'
```

### 2. 使用 Postman 测试

- **方法**: POST
- **URL**: `http://localhost:5000/webhook`
- **Headers**: `Content-Type: application/json`
- **Body**: 选择 raw JSON，粘贴上述示例数据

---

## 九、常见问题

### Q1: 为什么我配置了推送地址但没有收到消息？
**A**: 当前版本的 LiveBox 只有配置 UI，实际的 HTTP 推送功能尚未实现。需要按照第六章的说明，在消息处理函数中添加 HTTP POST 代码。

### Q2: 如何知道消息推送成功？
**A**: 可以在浏览器开发者工具的 Network 标签中查看 HTTP 请求，或在接收端程序中打印日志。

### Q3: 可以推送到远程服务器吗？
**A**: 可以，只要推送地址是可访问的 HTTP/HTTPS 地址即可。注意确保：
- 服务器防火墙开放对应端口
- 如果使用 HTTPS，确保证书有效
- 网络可达（可以先用 curl 测试）

### Q4: 消息的 msgId 是什么格式？
**A**: msgId 是 uint64 类型的数字，在 JavaScript 中可能会超出安全整数范围，建议转换为字符串处理。

### Q5: 如何只推送特定类型的消息？
**A**: 在 LiveBox 设置中勾选需要的消息类型，未勾选的类型不会被推送。

---

## 十、文件位置索引

| 文件 | 位置 | 说明 |
|------|------|------|
| 推送配置 UI | `src/App.vue:74-120` | 设置对话框 |
| 推送地址变量 | `src/App.vue:170` | pushUrl 定义 |
| 消息处理入口 | `src/App.vue:388-416` | onMessage 函数 |
| 消息分发 | `src/App.vue:419-475` | handleMessage 函数 |
| 聊天处理 | `src/App.vue:477-489` | decodeChat 函数 |
| 礼物处理 | `src/App.vue:491-503` | decodeGift 函数 |
| 点赞处理 | `src/App.vue:519-533` | likeLive 函数 |
| 关注处理 | `src/App.vue:536-549` | followLive 函数 |
| 进入处理 | `src/App.vue:506-516` | enterLive 函数 |
| Proto 定义 | `src/proto/dy.proto` | 所有消息结构定义 |

---

## 十一、参考链接

- LiveBox 项目: https://github.com/Sjj1024/LiveBox
- Protocol Buffers: https://protobuf.dev/
- 抖音直播开放平台: https://open.douyin.com/

---

**文档更新时间**: 2025-11-17
**适用版本**: LiveBox 当前版本
**维护者**: Claude
