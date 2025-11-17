#!/usr/bin/env python3
"""
抖音直播间聊天弹幕监控 - 手动签名版本
适用于无法使用 PyExecJS 的情况
"""

import re
import gzip
import json
import asyncio
import requests
from typing import Optional, Dict, Any
from urllib.parse import urlencode

try:
    import websockets
except ImportError:
    print("请先安装 websockets: pip install websockets")
    exit(1)

try:
    from dy_pb2 import PushFrame, Response, ChatMessage, GiftMessage, LikeMessage, MemberMessage, SocialMessage
    PROTOBUF_AVAILABLE = True
except ImportError:
    print("警告: 未找到 protobuf 定义，将使用原始数据模式")
    PROTOBUF_AVAILABLE = False


class DouyinLiveMonitor:
    """抖音直播间监控器 - 手动签名版本"""

    def __init__(self, live_url: str, manual_signature: str = ""):
        self.live_url = live_url
        self.room_id = None
        self.ttwid = None
        self.unique_id = None
        self.websocket = None
        self.running = False
        self.manual_signature = manual_signature  # 手动提供的签名

        # 回调函数
        self.on_chat = None
        self.on_gift = None
        self.on_like = None
        self.on_member = None
        self.on_follow = None

    def set_chat_callback(self, callback):
        """设置聊天消息回调"""
        self.on_chat = callback

    def get_room_info(self) -> Dict[str, Any]:
        """获取直播间信息"""
        print(f"正在获取直播间信息: {self.live_url}")

        headers = {
            'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
            'accept-language': 'zh-CN,zh;q=0.9',
            'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36',
        }

        try:
            response = requests.get(self.live_url, headers=headers, timeout=10)
            response.raise_for_status()

            self.ttwid = response.cookies.get('ttwid', '')
            html = response.text

            # 判断是否停播
            if 'status\\":4' in html:
                print("直播间已停播")
                pattern = r'anchor\\":(.*?),\\"open_id_str'
            else:
                print("直播间正在直播中")
                pattern = r'roomInfo\\":\{\\"room\\":(.*?),\\"toolbar_data'

                unique_pattern = r'user_unique_id\\":\\"(.*?)\\"}'
                unique_match = re.search(unique_pattern, html)
                if unique_match:
                    self.unique_id = unique_match.group(1)
                    print(f"获取到 unique_id: {self.unique_id}")

            match = re.search(pattern, html)
            if not match:
                raise Exception("无法从页面中提取直播间信息")

            room_info_str = match.group(1) + '}'
            room_info_str = room_info_str.replace('\\"', '"')

            room_info = json.loads(room_info_str)
            self.room_id = room_info.get('id_str', '')

            print(f"直播间 ID: {self.room_id}")
            print(f"主播昵称: {room_info.get('owner', {}).get('nickname', '未知')}")

            return {'room_info': room_info, 'ttwid': self.ttwid, 'unique_id': self.unique_id}

        except Exception as e:
            print(f"获取直播间信息失败: {e}")
            raise

    def build_websocket_url(self) -> str:
        """构建 WebSocket 连接 URL"""
        # 使用手动提供的签名
        signature = self.manual_signature

        params = {
            'room_id': self.room_id,
            'compress': 'gzip',
            'version_code': '180800',
            'webcast_sdk_version': '1.0.14-beta.0',
            'live_id': '1',
            'did_rule': '3',
            'user_unique_id': self.unique_id or '',
            'identity': 'audience',
            'signature': signature,
            'aid': '6383',
            'device_platform': 'web',
            'browser_language': 'zh-CN',
            'browser_platform': 'Win32',
            'browser_name': 'Mozilla',
            'browser_version': '5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        }

        base_url = 'wss://webcast5-ws-web-lf.douyin.com/webcast/im/push/v2/'
        ws_url = f"{base_url}?{urlencode(params)}"

        return ws_url

    def parse_message(self, data: bytes):
        """解析 WebSocket 消息"""
        try:
            if not PROTOBUF_AVAILABLE:
                print(f"接收到 {len(data)} 字节数据")
                return

            push_frame = PushFrame()
            push_frame.ParseFromString(data)

            decompressed = gzip.decompress(push_frame.payload)

            response = Response()
            response.ParseFromString(decompressed)

            if response.needAck:
                ack_frame = PushFrame()
                ack_frame.payloadType = 'ack'
                ack_frame.logId = push_frame.logId
                asyncio.create_task(self.websocket.send(ack_frame.SerializeToString()))

            for msg in response.messagesList:
                self.handle_message(msg)

        except Exception as e:
            print(f"解析消息失败: {e}")

    def handle_message(self, msg):
        """根据消息类型分发处理"""
        method = msg.method

        try:
            if method == 'WebcastChatMessage':
                chat_msg = ChatMessage()
                chat_msg.ParseFromString(msg.payload)

                data = {
                    'id': str(chat_msg.common.msgId),
                    'name': chat_msg.user.nickName,
                    'msg': chat_msg.content,
                }

                print(f"[聊天] {data['name']}: {data['msg']}")

                if self.on_chat:
                    self.on_chat(data)

            elif method == 'WebcastGiftMessage':
                gift_msg = GiftMessage()
                gift_msg.ParseFromString(msg.payload)
                print(f"[礼物] {gift_msg.user.nickName} 送出 {gift_msg.gift.name} x{gift_msg.repeatCount}")

            elif method == 'WebcastLikeMessage':
                like_msg = LikeMessage()
                like_msg.ParseFromString(msg.payload)
                print(f"[点赞] {like_msg.user.nickName} 点赞了")

        except Exception as e:
            print(f"处理消息失败 ({method}): {e}")

    async def connect(self):
        """连接到 WebSocket 服务器"""
        ws_url = self.build_websocket_url()

        headers = {
            'cookie': f'ttwid={self.ttwid}',
            'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        }

        print(f"\n正在连接 WebSocket...")
        print(f"使用签名: {self.manual_signature[:30]}...")

        try:
            try:
                async with websockets.connect(ws_url, additional_headers=headers) as websocket:
                    self.websocket = websocket
                    self.running = True
                    print("✓ WebSocket 连接成功！")
                    print("开始监听消息...\n")
                    print("=" * 60)

                    heartbeat_task = asyncio.create_task(self.heartbeat())

                    try:
                        async for message in websocket:
                            self.parse_message(message)
                    except websockets.exceptions.ConnectionClosed:
                        print("\nWebSocket 连接已关闭")
                    finally:
                        self.running = False
                        heartbeat_task.cancel()
            except TypeError:
                async with websockets.connect(ws_url, extra_headers=headers) as websocket:
                    self.websocket = websocket
                    self.running = True
                    print("✓ WebSocket 连接成功！")
                    print("开始监听消息...\n")
                    print("=" * 60)

                    heartbeat_task = asyncio.create_task(self.heartbeat())

                    try:
                        async for message in websocket:
                            self.parse_message(message)
                    except websockets.exceptions.ConnectionClosed:
                        print("\nWebSocket 连接已关闭")
                    finally:
                        self.running = False
                        heartbeat_task.cancel()

        except Exception as e:
            print(f"\n❌ WebSocket 连接失败: {e}")
            print("\n可能的原因:")
            print("1. 签名无效或已过期")
            print("2. 直播间不存在或已结束")
            print("3. 网络连接问题")
            print("\n解决方案:")
            print("- 重新从浏览器获取最新的签名")
            print("- 确认直播间正在直播中")
            print("- 检查网络连接")
            raise

    async def heartbeat(self):
        """发送心跳包"""
        while self.running:
            try:
                await asyncio.sleep(10)

                if PROTOBUF_AVAILABLE and self.websocket:
                    ping_frame = PushFrame()
                    ping_frame.payloadType = 'hb'
                    await self.websocket.send(ping_frame.SerializeToString())
                    print("♥ 心跳")

            except Exception as e:
                print(f"心跳发送失败: {e}")
                break

    def start(self):
        """启动监控"""
        print("=" * 60)
        print("抖音直播间聊天弹幕监控 - 手动签名版本")
        print("=" * 60)

        self.get_room_info()

        if not self.room_id:
            print("错误: 未能获取到直播间 ID")
            return

        asyncio.run(self.connect())


def normalize_url(url: str) -> str:
    """规范化 URL"""
    url = url.strip().strip('"').strip("'")

    if url.startswith('live.douyin.com'):
        url = 'https://' + url
    elif url.startswith('s://'):
        url = 'https' + url[1:]
    elif not url.startswith('https://'):
        if 'douyin.com' in url:
            url = 'https://' + url

    return url.replace('http://', 'https://')


def main():
    """主函数"""
    print("\n" + "=" * 60)
    print("抖音直播间弹幕监控 - 手动签名版本")
    print("=" * 60)
    print("\n这个版本需要你手动从浏览器获取签名")
    print("详细步骤请查看: QUICK_FIX_SIGNATURE.md\n")

    # 输入直播间 URL
    live_url = input("请输入抖音直播间 URL: ").strip()
    if not live_url:
        live_url = "https://live.douyin.com/816699487040"
        print(f"使用默认 URL: {live_url}")
    else:
        live_url = normalize_url(live_url)

    # 输入手动获取的签名
    print("\n" + "=" * 60)
    print("如何获取签名:")
    print("=" * 60)
    print("1. 在浏览器中打开上述直播间")
    print("2. 按 F12 打开开发者工具")
    print("3. 切换到 Network 标签，筛选 WS")
    print("4. 找到 im/push/v2/ 连接")
    print("5. 复制 URL 中的 signature= 参数的值")
    print("=" * 60 + "\n")

    signature = input("请粘贴签名 (signature=后面的部分): ").strip()

    if not signature:
        print("\n❌ 错误: 未提供签名")
        print("程序无法继续，请参考文档获取签名后重试")
        return

    # 创建监控器
    monitor = DouyinLiveMonitor(live_url, manual_signature=signature)

    # 启动监控
    try:
        monitor.start()
    except KeyboardInterrupt:
        print("\n\n用户中断，程序退出")
    except Exception as e:
        print(f"\n程序异常: {e}")


if __name__ == '__main__':
    main()
