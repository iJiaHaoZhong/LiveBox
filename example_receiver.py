#!/usr/bin/env python3
"""
LiveBox 推送消息接收服务器示例
用于接收来自 LiveBox 的直播消息推送
"""

from flask import Flask, request, jsonify
from datetime import datetime
import json

app = Flask(__name__)

# 消息处理器字典
message_handlers = {}


def register_handler(msg_type):
    """装饰器：注册消息处理器"""
    def decorator(func):
        message_handlers[msg_type] = func
        return func
    return decorator


@register_handler('chat')
def handle_chat(data):
    """处理聊天消息"""
    print(f"[聊天] {data['name']}: {data['msg']}")
    # 在这里添加你的业务逻辑
    # 例如：保存到数据库、触发关键词回复等
    return {"status": "ok", "action": "chat_received"}


@register_handler('gift')
def handle_gift(data):
    """处理礼物消息"""
    print(f"[礼物] {data['name']} {data['msg']}")
    # 在这里添加你的业务逻辑
    # 例如：统计礼物收益、触发感谢语等
    return {"status": "ok", "action": "gift_received"}


@register_handler('like')
def handle_like(data):
    """处理点赞消息"""
    print(f"[点赞] {data['name']} {data['msg']}")
    # 在这里添加你的业务逻辑
    return {"status": "ok", "action": "like_received"}


@register_handler('follow')
def handle_follow(data):
    """处理关注消息"""
    print(f"[关注] {data['name']} {data['msg']}")
    # 在这里添加你的业务逻辑
    # 例如：欢迎新粉丝、记录关注列表等
    return {"status": "ok", "action": "follow_received"}


@register_handler('comein')
def handle_comein(data):
    """处理进入直播间消息"""
    print(f"[进入] {data['name']} {data['msg']}")
    # 在这里添加你的业务逻辑
    # 例如：欢迎用户、统计在线人数等
    return {"status": "ok", "action": "comein_received"}


@app.route('/webhook', methods=['POST'])
def webhook():
    """
    接收 LiveBox 推送的消息
    预期的消息格式：
    {
        "type": "chat|gift|like|follow|comein",
        "data": {
            "id": "消息ID",
            "name": "用户昵称",
            "msg": "消息内容"
        },
        "raw": { ... }  # 原始的完整数据
    }
    """
    try:
        # 获取请求数据
        message = request.get_json()

        if not message:
            return jsonify({"error": "No JSON data received"}), 400

        # 打印接收到的完整消息（调试用）
        print(f"\n{'='*60}")
        print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] 收到消息")
        print(f"消息类型: {message.get('type', 'unknown')}")
        print(f"消息内容: {json.dumps(message, ensure_ascii=False, indent=2)}")
        print(f"{'='*60}\n")

        # 获取消息类型
        msg_type = message.get('type')

        if not msg_type:
            return jsonify({"error": "Missing message type"}), 400

        # 获取消息数据
        data = message.get('data', {})

        # 调用对应的处理器
        if msg_type in message_handlers:
            result = message_handlers[msg_type](data)
            return jsonify(result), 200
        else:
            print(f"[警告] 未知的消息类型: {msg_type}")
            return jsonify({"error": f"Unknown message type: {msg_type}"}), 400

    except Exception as e:
        print(f"[错误] 处理消息时发生异常: {str(e)}")
        import traceback
        traceback.print_exc()
        return jsonify({"error": str(e)}), 500


@app.route('/health', methods=['GET'])
def health():
    """健康检查端点"""
    return jsonify({
        "status": "ok",
        "service": "LiveBox Message Receiver",
        "timestamp": datetime.now().isoformat()
    })


@app.route('/', methods=['GET'])
def index():
    """根路径信息"""
    return jsonify({
        "service": "LiveBox Message Receiver",
        "endpoints": {
            "webhook": "/webhook (POST)",
            "health": "/health (GET)"
        },
        "supported_message_types": list(message_handlers.keys())
    })


if __name__ == '__main__':
    print("="*60)
    print("LiveBox 消息接收服务器启动")
    print("="*60)
    print(f"Webhook 地址: http://localhost:5000/webhook")
    print(f"健康检查: http://localhost:5000/health")
    print(f"支持的消息类型: {', '.join(message_handlers.keys())}")
    print("="*60)
    print("\n在 LiveBox 中配置推送地址为: http://localhost:5000/webhook\n")

    # 启动服务器
    # 0.0.0.0 允许外部访问，如果只在本地测试可以改为 127.0.0.1
    app.run(host='0.0.0.0', port=5000, debug=True)
