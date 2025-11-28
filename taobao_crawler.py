#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import asyncio
import json
import time
import argparse
import base64
import binascii
import zlib
import re
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

from playwright.async_api import async_playwright, Browser, Page
from loguru import logger
import aiohttp


class TaobaoCrawler:
    """淘宝直播间弹幕抓取器"""

    def __init__(self, room_id: str, output_file: str = None, push_url: str = None):
        self.room_id = room_id
        self.output_file = output_file or f"taobao_{room_id}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        self.comments = []
        self.browser: Optional[Browser] = None
        self.page: Optional[Page] = None
        self.push_url = push_url

        # 设置日志
        logger.add(f"logs/taobao_{room_id}.log", rotation="1 day", retention="7 days")

        if self.push_url:
            logger.info(f"推送地址已配置: {self.push_url}")
        
    async def start(self):
        """启动抓取器"""
        try:
            async with async_playwright() as p:
                # 启动浏览器
                self.browser = await p.chromium.launch(
                    headless=False,  # 默认显示浏览器窗口，必要时可改为 True
                    args=[
                        '--disable-blink-features=AutomationControlled',
                        '--disable-web-security',
                        '--disable-features=VizDisplayCompositor'
                    ]
                )
                
                # 创建新页面
                self.page = await self.browser.new_page()
                
                # 设置用户代理
                await self.page.set_extra_http_headers({
                    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0'
                })
                
                # 注册弹幕监听
                await self._setup_comment_listeners()
                
                # 访问直播间
                await self._navigate_to_live_room()
                
                # 开始监听弹幕
                await self._start_comment_monitoring()
                
        except Exception as e:
            logger.error(f"启动失败: {e}")
            raise
    
    async def _navigate_to_live_room(self):
        """导航到直播间"""
        try:
            # 仅通过 tbzb.taobao.com 入口访问直播间
            live_url = f"https://tbzb.taobao.com/live?liveId={self.room_id}"
            logger.info(f"正在访问直播间: {live_url}")
            
            await self.page.goto(
                live_url,
                wait_until="domcontentloaded",
                timeout=60000
            )
            
            await asyncio.sleep(6)
            await self._check_live_room_status()
            
        except Exception as e:
            logger.error(f"导航到直播间失败: {e}")
            raise

    async def _check_live_room_status(self):
        """检查直播间状态"""
        try:
            # 检查是否存在直播结束提示
            end_text = await self.page.query_selector('text=直播已结束')
            if end_text:
                logger.warning("直播间已结束")
                return False
            
            # 检查是否存在弹幕区域
            comment_area = await self.page.query_selector('.chat-container')
            if not comment_area:
                comment_area = await self.page.query_selector('#liveComment')
            if not comment_area:
                logger.warning("未找到弹幕区域，可能需要手动处理（chat-container/liveComment）")
            
            logger.info("直播间加载成功")
            return True
            
        except Exception as e:
            logger.error(f"检查直播间状态失败: {e}")
            return False
    
    async def _setup_comment_listeners(self):
        """注册弹幕抓取相关监听"""
        try:
            await self.page.route("**/*", self._handle_network_request)
            self.page.on("websocket", self._handle_websocket)
            self.page.on("console", self._handle_console_message)
            logger.info("弹幕监听钩子注册完成")
        except Exception as e:
            logger.error(f"注册弹幕监听失败: {e}")
            raise

    async def _start_comment_monitoring(self):
        """��ʼ������Ļ"""
        logger.info("��ʼ������Ļ...")

        try:
            while True:
                await asyncio.sleep(1)

                # ���ڱ�������
                if len(self.comments) % 10 == 0 and self.comments:
                    await self._save_comments()

        except KeyboardInterrupt:
            logger.info("�յ��ж��źţ�����ֹͣ...")
        except Exception as e:
            logger.error(f"监听弹幕时发生错误: {e}")
        finally:
            await self._cleanup()

    async def _handle_network_request(self, route):
        """处理网络请求，提取弹幕数据"""
        try:
            request = route.request
            
            url_lower = request.url.lower()
            keyword_tokens = ("comment", "danmu", "chat", "msg", "message")
            if not any(token in url_lower for token in keyword_tokens):
                await route.continue_()
                return
            
            api_patterns = [
                "live.taobao.com/api/chat",
                "live.taobao.com/api/message",
                "live.taobao.com/api/comment",
                "mtop.taobao.iliad.comment",
                "mtop.taobao.powermsg",
                "mtop.tblive",
                "taobao.com/live/message"
            ]
            if not any(pattern in url_lower for pattern in api_patterns):
                await route.continue_()
                return
            
            logger.info(f"拦截到弹幕接口: {request.url}")
            response = await route.fetch()
            
            if response.ok:
                payload = None
                try:
                    payload = await response.json()
                except Exception:
                    try:
                        text = await response.text()
                        payload = self._parse_jsonp_text(text)
                    except Exception:
                        payload = None
                if payload is None:
                    logger.warning(f"未能解析接口响应: {request.url}")
                else:
                    await self._extract_comments_from_response(payload)
                        
            await route.continue_()
            
        except Exception as e:
            logger.error(f"处理网络请求失败: {e}")
            await route.continue_()
    
    async def _handle_websocket(self, websocket):
        """处理WebSocket连接"""
        logger.info(f"WebSocket连接: {websocket.url}")
        if "mmstat.com" in websocket.url:
            logger.info("检测到埋点 WebSocket，跳过处理")
            return
        
        async def handle_message(msg):
            try:
                payloads = self._parse_websocket_payloads(msg)
                for payload in payloads:
                    await self._extract_comments_from_websocket(payload)
            except Exception as e:
                logger.error(f"处理WebSocket消息失败: {e}")
        
        websocket.on("framesent", handle_message)
        websocket.on("framereceived", handle_message)
    
    def _parse_websocket_payloads(self, msg) -> List[Dict]:
        payloads = []
        for text in self._generate_text_candidates(msg):
            for chunk in self._split_json_chunks(text):
                if not chunk:
                    continue
                try:
                    payloads.append(json.loads(chunk))
                except json.JSONDecodeError:
                    continue
        
        if not payloads:
            preview = self._get_payload_preview(msg)
            logger.warning(
                f"无法解析 WebSocket 消息，type={getattr(msg, 'type', None)} "
                f"opcode={getattr(msg, 'opcode', None)} preview={preview}"
            )
        return payloads
    
    def _generate_text_candidates(self, msg) -> List[str]:
        texts = []
        payload = self._extract_frame_payload(msg)
        if self._is_text_frame(msg, payload):
            text = payload if isinstance(payload, str) else self._decode_bytes_to_text(payload)
            if text:
                texts.append(text)
                decoded = self._decode_base64_text(text)
                if decoded:
                    texts.append(decoded)
        elif self._is_binary_frame(msg, payload):
            decoded_text = self._decode_bytes_to_text(payload)
            if decoded_text:
                texts.append(decoded_text)
        else:
            # 回退策略：尝试把 payload 当文本或二进制处理
            if isinstance(payload, str):
                texts.append(payload)
                decoded = self._decode_base64_text(payload)
                if decoded:
                    texts.append(decoded)
            else:
                decoded_text = self._decode_bytes_to_text(payload)
                if decoded_text:
                    texts.append(decoded_text)
        return texts
    
    def _extract_frame_payload(self, msg):
        for attr in ("payload", "text", "data"):
            value = getattr(msg, attr, None)
            if value is not None:
                return value
        if hasattr(msg, "json"):
            return msg.json
        return None
    
    def _is_text_frame(self, msg, payload) -> bool:
        frame_type = getattr(msg, "type", None)
        if frame_type == "text":
            return True
        opcode = getattr(msg, "opcode", None)
        if opcode == 1:
            return True
        return isinstance(payload, str)
    
    def _is_binary_frame(self, msg, payload) -> bool:
        frame_type = getattr(msg, "type", None)
        if frame_type == "binary":
            return True
        opcode = getattr(msg, "opcode", None)
        if opcode == 2:
            return True
        return isinstance(payload, (bytes, bytearray, memoryview))
    
    def _decode_base64_text(self, text: str) -> Optional[str]:
        try:
            raw = base64.b64decode(text)
        except binascii.Error:
            return None
        return self._decode_bytes_to_text(raw)
    
    def _decode_bytes_to_text(self, data) -> Optional[str]:
        if not data:
            return None
        if isinstance(data, str):
            return data
        
        raw = bytes(data)
        buffers = [raw]
        if len(raw) > 4:
            buffers.append(raw[4:])
        if len(raw) > 8:
            buffers.append(raw[8:])
        
        candidates = []
        for buffer in buffers:
            candidates.append(buffer)
            for wbits in (16 + zlib.MAX_WBITS, zlib.MAX_WBITS, 0):
                try:
                    decompressed = zlib.decompress(buffer, wbits)
                    candidates.append(decompressed)
                except zlib.error:
                    continue
        
        for candidate in candidates:
            try:
                return candidate.decode("utf-8")
            except UnicodeDecodeError:
                continue
        return None
    
    def _split_json_chunks(self, text: str) -> List[str]:
        if not text:
            return []
        chunks = [text]
        for sep in ("\x1e", "\n"):
            next_chunks = []
            for chunk in chunks:
                next_chunks.extend(part.strip() for part in chunk.split(sep))
            chunks = [c for c in next_chunks if c]
        return chunks
    
    def _get_payload_preview(self, msg) -> str:
        if getattr(msg, "type", None) == "text":
            text = getattr(msg, "text", "") or ""
            return text[:200]
        data = getattr(msg, "data", None)
        if data is None and hasattr(msg, "payload"):
            data = msg.payload
        if data is None and hasattr(msg, "text"):
            data = msg.text
        if data:
            return str(data)[:200]
        return ""

    async def _handle_console_message(self, msg):
        """处理控制台消息"""
        try:
            if "弹幕" in msg.text or "chat" in msg.text.lower():
                logger.info(f"控制台消息: {msg.text}")
        except Exception as e:
            logger.error(f"处理控制台消息失败: {e}")
    
    async def _extract_comments_from_response(self, data: Dict):
        """从 API 响应中提取弹幕"""
        try:
            for message in self._iter_comment_candidates(data):
                content = self._extract_comment_content(message)
                if not content:
                    continue
                user = self._extract_comment_user(message) or "未知用户"
                comment_type = str(
                    message.get("type") or
                    message.get("msgType") or
                    message.get("bizType") or
                    message.get("cmd") or
                    "chat"
                )
                comment = {
                    "timestamp": datetime.now().isoformat(),
                    "user": user,
                    "content": content,
                    "type": comment_type
                }
                self.comments.append(comment)
                logger.info(f"弹幕: {comment['user']}: {comment['content']}")

                # 推送到配置的URL
                if self.push_url:
                    await self._push_comment(comment)
        except Exception as e:
            logger.error(f"解析弹幕失败: {e}")

    async def _extract_comments_from_websocket(self, data):
        """从WebSocket消息中提取弹幕"""
        try:
            if data is None:
                return
            if isinstance(data, str):
                try:
                    data = json.loads(data)
                except json.JSONDecodeError:
                    return
            if isinstance(data, list):
                for item in data:
                    await self._extract_comments_from_websocket(item)
                return
            if not isinstance(data, dict):
                return
            
            nested_handled = False
            for key in ("payload", "body"):
                payload = data.get(key)
                if isinstance(payload, (dict, list)):
                    await self._extract_comments_from_websocket(payload)
                    nested_handled = True
                elif isinstance(payload, str):
                    try:
                        payload_data = json.loads(payload)
                        await self._extract_comments_from_websocket(payload_data)
                        nested_handled = True
                    except json.JSONDecodeError:
                        continue
            
            if "data" in data and isinstance(data["data"], (dict, list)):
                await self._extract_comments_from_websocket(data["data"])
                nested_handled = True
            
            if "messages" in data and isinstance(data["messages"], list):
                for message in data["messages"]:
                    await self._extract_comments_from_websocket(message)
                nested_handled = True
            
            if nested_handled:
                return
            
            content = self._extract_comment_content(data)
            if not content:
                return
            
            user = self._extract_comment_user(data) or "未知用户"
            comment_type = str(
                data.get("type") or
                data.get("msgType") or
                data.get("bizType") or
                data.get("cmd") or
                "chat"
            )
            
            comment = {
                "timestamp": datetime.now().isoformat(),
                "user": user,
                "content": content,
                "type": comment_type
            }
            self.comments.append(comment)
            logger.info(f"弹幕: {comment['user']}: {comment['content']}")

            # 推送到配置的URL
            if self.push_url:
                await self._push_comment(comment)
                
        except Exception as e:
            logger.error(f"从WebSocket提取弹幕失败: {e}")
    
    def _extract_comment_content(self, message: Dict) -> Optional[str]:
        content_keys = ["content", "text", "msg", "message", "contentText", "commentContent"]
        for key in content_keys:
            value = message.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
            if isinstance(value, dict):
                for subkey in ("text", "content", "value", "string"):
                    subval = value.get(subkey)
                    if isinstance(subval, str) and subval.strip():
                        return subval.strip()
            if isinstance(value, list):
                for item in value:
                    if isinstance(item, str) and item.strip():
                        return item.strip()
        
        ext = message.get("ext")
        if isinstance(ext, dict):
            for key in ("commentContent", "content", "msg"):
                value = ext.get(key)
                if isinstance(value, str) and value.strip():
                    return value.strip()
        
        payload = message.get("payload")
        if isinstance(payload, dict):
            return self._extract_comment_content(payload)
        
        return None
    
    def _extract_comment_user(self, message: Dict) -> Optional[str]:
        user_sources = []
        for key in ("user", "fromUser", "sender", "author", "userInfo", "authorInfo"):
            val = message.get(key)
            if isinstance(val, dict):
                user_sources.append(val)
        
        for source in user_sources:
            for key in ("nickname", "nick", "name", "userNick", "senderNick", "displayName"):
                value = source.get(key)
                if value:
                    return value
        
        ext = message.get("ext")
        if isinstance(ext, dict):
            for key in ("nick", "senderNick", "userNick"):
                value = ext.get(key)
                if value:
                    return value
        
        for key in ("nick", "fromNick", "userName", "displayName"):
            value = message.get(key)
            if value:
                return value
        
        return None

    def _iter_comment_candidates(self, payload):
        stack = [payload]
        while stack:
            item = stack.pop()
            if isinstance(item, dict):
                yield item
                for value in item.values():
                    if isinstance(value, (dict, list)):
                        stack.append(value)
            elif isinstance(item, list):
                stack.extend(item)

    def _parse_jsonp_text(self, text: str):
        if not text:
            return None
        stripped = text.strip()
        if not stripped:
            return None
        if stripped[0] in "{[":
            try:
                return json.loads(stripped)
            except json.JSONDecodeError:
                return None
        match = re.match(r"^[^(]+\((.*)\)\s*$", stripped, re.S)
        if not match:
            return None
        body = match.group(1).strip()
        if not body:
            return None
        try:
            return json.loads(body)
        except json.JSONDecodeError:
            return None
    
    async def _push_comment(self, comment: Dict):
        """推送弹幕到配置的URL"""
        try:
            async with aiohttp.ClientSession() as session:
                payload = {
                    "type": "chat",
                    "platform": "taobao",
                    "data": {
                        "id": str(hash(f"{comment['timestamp']}{comment['user']}{comment['content']}")),
                        "name": comment["user"],
                        "msg": comment["content"]
                    },
                    "raw": comment,
                    "timestamp": int(datetime.now().timestamp() * 1000),
                    "room_id": self.room_id
                }

                async with session.post(
                    self.push_url,
                    json=payload,
                    headers={"Content-Type": "application/json"},
                    timeout=aiohttp.ClientTimeout(total=5)
                ) as response:
                    if response.status == 200:
                        logger.debug(f"弹幕推送成功")
                    else:
                        logger.warning(f"弹幕推送失败: HTTP {response.status}")
        except asyncio.TimeoutError:
            logger.warning("弹幕推送超时")
        except Exception as e:
            logger.error(f"推送弹幕失败: {e}")

    async def _save_comments(self):
        """保存弹幕数据"""
        try:
            # 确保输出目录存在
            output_path = Path(self.output_file)
            output_path.parent.mkdir(parents=True, exist_ok=True)

            # 保存为JSON格式
            with open(self.output_file, 'w', encoding='utf-8') as f:
                json.dump(self.comments, f, ensure_ascii=False, indent=2)

            logger.info(f"已保存 {len(self.comments)} 条弹幕到 {self.output_file}")

        except Exception as e:
            logger.error(f"保存弹幕失败: {e}")
    
    async def _cleanup(self):
        """清理资源"""
        try:
            if self.page:
                await self.page.close()
            if self.browser:
                await self.browser.close()
            
            # 最终保存
            await self._save_comments()
            logger.info("清理完成")
            
        except Exception as e:
            logger.error(f"清理失败: {e}")


async def main():
    """主函数"""
    parser = argparse.ArgumentParser(description="淘宝直播间弹幕抓取工具")
    parser.add_argument("--room_id", required=True, help="直播间ID")
    parser.add_argument("--output", help="输出文件路径")
    parser.add_argument("--push_url", help="推送弹幕的URL地址")

    args = parser.parse_args()

    # 创建日志目录
    Path("logs").mkdir(exist_ok=True)

    # 创建抓取器并启动
    crawler = TaobaoCrawler(args.room_id, args.output, args.push_url)
    await crawler.start()


if __name__ == "__main__":
    asyncio.run(main()) 



