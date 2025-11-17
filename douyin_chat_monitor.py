#!/usr/bin/env python3
"""
抖音直播间聊天弹幕监控 - Python 实现
基于 LiveBox 项目的原理实现
"""

import re
import gzip
import json
import time
import hashlib
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
    # 需要先编译 proto 文件生成 Python 代码
    # protoc --python_out=. dy.proto
    from dy_pb2 import PushFrame, Response, ChatMessage, GiftMessage, LikeMessage, MemberMessage, SocialMessage
    PROTOBUF_AVAILABLE = True
except ImportError:
    print("警告: 未找到 protobuf 定义，将使用原始数据模式")
    print("要使用完整功能，请运行: protoc --python_out=. src/proto/dy.proto")
    PROTOBUF_AVAILABLE = False


class DouyinLiveMonitor:
    """抖音直播间监控器"""

    def __init__(self, live_url: str):
        self.live_url = live_url
        self.room_id = None
        self.ttwid = None
        self.unique_id = None
        self.websocket = None
        self.running = False

        # 回调函数
        self.on_chat = None
        self.on_gift = None
        self.on_like = None
        self.on_member = None
        self.on_follow = None

    def set_chat_callback(self, callback):
        """设置聊天消息回调"""
        self.on_chat = callback

    def set_gift_callback(self, callback):
        """设置礼物消息回调"""
        self.on_gift = callback

    def set_like_callback(self, callback):
        """设置点赞消息回调"""
        self.on_like = callback

    def set_member_callback(self, callback):
        """设置进入房间消息回调"""
        self.on_member = callback

    def set_follow_callback(self, callback):
        """设置关注消息回调"""
        self.on_follow = callback

    def get_room_info(self) -> Dict[str, Any]:
        """
        获取直播间信息
        模拟 LiveBox 的 Rust 后端逻辑
        """
        print(f"正在获取直播间信息: {self.live_url}")

        headers = {
            'accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8',
            'accept-language': 'zh-CN,zh;q=0.9,en;q=0.8',
            'cache-control': 'max-age=0',
            'sec-ch-ua': '"Chromium";v="124", "Google Chrome";v="124", "Not-A.Brand";v="99"',
            'sec-ch-ua-mobile': '?0',
            'sec-ch-ua-platform': '"Windows"',
            'sec-fetch-dest': 'document',
            'sec-fetch-mode': 'navigate',
            'sec-fetch-site': 'none',
            'upgrade-insecure-requests': '1',
            'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36',
        }

        try:
            response = requests.get(self.live_url, headers=headers, timeout=10)
            response.raise_for_status()

            # 提取 ttwid Cookie
            self.ttwid = response.cookies.get('ttwid', '')
            if not self.ttwid:
                print("警告: 未获取到 ttwid Cookie")

            html = response.text

            # 判断是否停播
            if 'status\\":4' in html:
                print("直播间已停播")
                # 提取主播信息（停播状态）
                pattern = r'anchor\\":(.*?),\\"open_id_str'
            else:
                print("直播间正在直播中")
                # 提取直播间信息（直播中）
                pattern = r'roomInfo\\":\{\\"room\\":(.*?),\\"toolbar_data'

                # 提取 user_unique_id
                unique_pattern = r'user_unique_id\\":\\"(.*?)\\"}'
                unique_match = re.search(unique_pattern, html)
                if unique_match:
                    self.unique_id = unique_match.group(1)
                    print(f"获取到 unique_id: {self.unique_id}")

            # 提取主要信息
            match = re.search(pattern, html)
            if not match:
                raise Exception("无法从页面中提取直播间信息")

            room_info_str = match.group(1) + '}'
            room_info_str = room_info_str.replace('\\"', '"')

            room_info = json.loads(room_info_str)
            self.room_id = room_info.get('id_str', '')

            print(f"直播间 ID: {self.room_id}")
            print(f"主播昵称: {room_info.get('owner', {}).get('nickname', '未知')}")

            return {
                'room_info': room_info,
                'ttwid': self.ttwid,
                'unique_id': self.unique_id,
            }

        except Exception as e:
            print(f"获取直播间信息失败: {e}")
            raise

    def generate_signature(self) -> str:
        """
        生成 WebSocket 连接签名

        注意: 这是一个简化版本，实际的签名算法依赖于抖音的 JavaScript 加密库
        完整实现需要使用 execjs 或其他方式调用 JavaScript 代码

        这里提供一个基础的 MD5 签名实现作为示例
        """
        # 构建签名字符串（与 LiveBox 中的逻辑一致）
        sign_str = f"live_id=1,aid=6383,version_code=180800,webcast_sdk_version=1.0.14-beta.0,room_id={self.room_id},sub_room_id=,sub_channel_id=,did_rule=3,user_unique_id={self.unique_id},device_platform=web,device_type=,ac=,identity=audience"

        # MD5 哈希
        md5_hash = hashlib.md5(sign_str.encode()).hexdigest()

        # 注意: 真实的签名还需要调用 byted_acrawler.frontierSign()
        # 这里返回的是简化版本，可能无法直接使用
        # 完整实现请参考 src/assets/static/vFun.js:166-193

        print(f"警告: 使用简化签名，可能需要完整的 JavaScript 签名算法")
        print(f"MD5 Hash: {md5_hash}")

        # 可以尝试使用固定的签名或者留空
        # 实际使用时可能需要通过其他方式获取有效签名
        return ""  # 返回空签名，某些情况下可能也能工作

    def build_websocket_url(self) -> str:
        """构建 WebSocket 连接 URL"""
        signature = self.generate_signature()

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
                print(f"接收到 {len(data)} 字节数据（未解析，需要 protobuf）")
                return

            # 1. 解码 PushFrame
            push_frame = PushFrame()
            push_frame.ParseFromString(data)

            # 2. gzip 解压
            decompressed = gzip.decompress(push_frame.payload)

            # 3. 解码 Response
            response = Response()
            response.ParseFromString(decompressed)

            # 4. 处理 ACK
            if response.needAck:
                ack_frame = PushFrame()
                ack_frame.payloadType = 'ack'
                ack_frame.logId = push_frame.logId
                asyncio.create_task(self.websocket.send(ack_frame.SerializeToString()))

            # 5. 遍历消息列表
            for msg in response.messagesList:
                self.handle_message(msg)

        except Exception as e:
            print(f"解析消息失败: {e}")

    def handle_message(self, msg):
        """根据消息类型分发处理"""
        method = msg.method

        try:
            if method == 'WebcastChatMessage':
                # 聊天消息
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
                # 礼物消息
                gift_msg = GiftMessage()
                gift_msg.ParseFromString(msg.payload)

                data = {
                    'id': str(gift_msg.common.msgId),
                    'name': gift_msg.user.nickName,
                    'gift_name': gift_msg.gift.name,
                    'gift_count': gift_msg.repeatCount,
                    'diamond_count': gift_msg.gift.diamondCount,
                }

                print(f"[礼物] {data['name']} 送出 {data['gift_name']} x{data['gift_count']}")

                if self.on_gift:
                    self.on_gift(data)

            elif method == 'WebcastLikeMessage':
                # 点赞消息
                like_msg = LikeMessage()
                like_msg.ParseFromString(msg.payload)

                data = {
                    'id': str(like_msg.common.msgId),
                    'name': like_msg.user.nickName,
                    'count': like_msg.count,
                    'total': like_msg.total,
                }

                print(f"[点赞] {data['name']} 点赞了 ({data['count']})")

                if self.on_like:
                    self.on_like(data)

            elif method == 'WebcastMemberMessage':
                # 进入房间消息
                member_msg = MemberMessage()
                member_msg.ParseFromString(msg.payload)

                data = {
                    'id': str(member_msg.common.msgId),
                    'name': member_msg.user.nickName,
                    'member_count': member_msg.memberCount,
                }

                print(f"[进入] {data['name']} 来了")

                if self.on_member:
                    self.on_member(data)

            elif method == 'WebcastSocialMessage':
                # 关注消息
                social_msg = SocialMessage()
                social_msg.ParseFromString(msg.payload)

                data = {
                    'id': str(social_msg.common.msgId),
                    'name': social_msg.user.nickName,
                    'follow_count': social_msg.followCount,
                }

                print(f"[关注] {data['name']} 关注了主播")

                if self.on_follow:
                    self.on_follow(data)

            else:
                # 其他消息类型
                pass

        except Exception as e:
            print(f"处理消息失败 ({method}): {e}")

    async def connect(self):
        """连接到 WebSocket 服务器"""
        ws_url = self.build_websocket_url()

        headers = {
            'cookie': f'ttwid={self.ttwid}',
            'user-agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36',
        }

        print(f"\n正在连接 WebSocket...")
        print(f"URL: {ws_url[:100]}...")

        try:
            async with websockets.connect(ws_url, extra_headers=headers) as websocket:
                self.websocket = websocket
                self.running = True
                print("✓ WebSocket 连接成功！")
                print("开始监听消息...\n")
                print("=" * 60)

                # 创建心跳任务
                heartbeat_task = asyncio.create_task(self.heartbeat())

                # 接收消息
                try:
                    async for message in websocket:
                        self.parse_message(message)
                except websockets.exceptions.ConnectionClosed:
                    print("\nWebSocket 连接已关闭")
                finally:
                    self.running = False
                    heartbeat_task.cancel()

        except Exception as e:
            print(f"WebSocket 连接失败: {e}")
            raise

    async def heartbeat(self):
        """发送心跳包"""
        while self.running:
            try:
                await asyncio.sleep(10)  # 每 10 秒发送一次

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
        print("抖音直播间聊天弹幕监控")
        print("=" * 60)

        # 1. 获取直播间信息
        self.get_room_info()

        if not self.room_id:
            print("错误: 未能获取到直播间 ID")
            return

        # 2. 连接 WebSocket
        asyncio.run(self.connect())


def main():
    """主函数示例"""
    # 直播间 URL（请替换为实际的直播间地址）
    live_url = input("请输入抖音直播间 URL: ").strip()

    if not live_url:
        print("使用默认测试 URL")
        live_url = "https://live.douyin.com/972176515698"

    # 创建监控器
    monitor = DouyinLiveMonitor(live_url)

    # 设置回调函数（可选）
    def on_chat_message(data):
        """聊天消息回调"""
        # 这里可以自定义处理逻辑，比如保存到数据库、推送到其他服务等
        pass

    monitor.set_chat_callback(on_chat_message)

    # 启动监控
    try:
        monitor.start()
    except KeyboardInterrupt:
        print("\n\n用户中断，程序退出")
    except Exception as e:
        print(f"\n程序异常: {e}")


if __name__ == '__main__':
    main()
