#!/usr/bin/env node
/**
 * LiveBox 推送消息接收服务器示例 (Node.js)
 * 用于接收来自 LiveBox 的直播消息推送
 */

const express = require('express');
const app = express();
const PORT = 3000;

// 解析 JSON 请求体
app.use(express.json());

// 消息处理器
const messageHandlers = {
    // 聊天消息处理
    chat: (data) => {
        console.log(`[聊天] ${data.name}: ${data.msg}`);
        // 在这里添加你的业务逻辑
        // 例如：保存到数据库、触发关键词回复等
        return { status: 'ok', action: 'chat_received' };
    },

    // 礼物消息处理
    gift: (data) => {
        console.log(`[礼物] ${data.name} ${data.msg}`);
        // 在这里添加你的业务逻辑
        // 例如：统计礼物收益、触发感谢语等
        return { status: 'ok', action: 'gift_received' };
    },

    // 点赞消息处理
    like: (data) => {
        console.log(`[点赞] ${data.name} ${data.msg}`);
        // 在这里添加你的业务逻辑
        return { status: 'ok', action: 'like_received' };
    },

    // 关注消息处理
    follow: (data) => {
        console.log(`[关注] ${data.name} ${data.msg}`);
        // 在这里添加你的业务逻辑
        // 例如：欢迎新粉丝、记录关注列表等
        return { status: 'ok', action: 'follow_received' };
    },

    // 进入直播间消息处理
    comein: (data) => {
        console.log(`[进入] ${data.name} ${data.msg}`);
        // 在这里添加你的业务逻辑
        // 例如：欢迎用户、统计在线人数等
        return { status: 'ok', action: 'comein_received' };
    }
};

/**
 * Webhook 端点 - 接收 LiveBox 推送的消息
 * POST /webhook
 *
 * 预期的消息格式：
 * {
 *     "type": "chat|gift|like|follow|comein",
 *     "data": {
 *         "id": "消息ID",
 *         "name": "用户昵称",
 *         "msg": "消息内容"
 *     },
 *     "raw": { ... }  // 原始的完整数据
 * }
 */
app.post('/webhook', (req, res) => {
    try {
        const message = req.body;

        if (!message) {
            return res.status(400).json({ error: 'No JSON data received' });
        }

        // 打印接收到的完整消息（调试用）
        console.log('\n' + '='.repeat(60));
        console.log(`[${new Date().toLocaleString()}] 收到消息`);
        console.log(`消息类型: ${message.type || 'unknown'}`);
        console.log(`消息内容: ${JSON.stringify(message, null, 2)}`);
        console.log('='.repeat(60) + '\n');

        // 获取消息类型
        const msgType = message.type;

        if (!msgType) {
            return res.status(400).json({ error: 'Missing message type' });
        }

        // 获取消息数据
        const data = message.data || {};

        // 调用对应的处理器
        if (messageHandlers[msgType]) {
            const result = messageHandlers[msgType](data);
            return res.json(result);
        } else {
            console.log(`[警告] 未知的消息类型: ${msgType}`);
            return res.status(400).json({ error: `Unknown message type: ${msgType}` });
        }

    } catch (error) {
        console.error(`[错误] 处理消息时发生异常:`, error);
        return res.status(500).json({ error: error.message });
    }
});

/**
 * 健康检查端点
 * GET /health
 */
app.get('/health', (req, res) => {
    res.json({
        status: 'ok',
        service: 'LiveBox Message Receiver',
        timestamp: new Date().toISOString()
    });
});

/**
 * 根路径信息
 * GET /
 */
app.get('/', (req, res) => {
    res.json({
        service: 'LiveBox Message Receiver',
        endpoints: {
            webhook: '/webhook (POST)',
            health: '/health (GET)'
        },
        supported_message_types: Object.keys(messageHandlers)
    });
});

// 启动服务器
app.listen(PORT, '0.0.0.0', () => {
    console.log('='.repeat(60));
    console.log('LiveBox 消息接收服务器启动');
    console.log('='.repeat(60));
    console.log(`Webhook 地址: http://localhost:${PORT}/webhook`);
    console.log(`健康检查: http://localhost:${PORT}/health`);
    console.log(`支持的消息类型: ${Object.keys(messageHandlers).join(', ')}`);
    console.log('='.repeat(60));
    console.log(`\n在 LiveBox 中配置推送地址为: http://localhost:${PORT}/webhook\n`);
});

// 优雅退出
process.on('SIGINT', () => {
    console.log('\n正在关闭服务器...');
    process.exit(0);
});
