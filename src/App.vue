<template>
    <div class="container">
        <!-- 顶部输入直播间地址 -->
        <div class="liveUrl">
            <input
                class="urlInput"
                v-model="inputUrl"
                placeholder="请输入直播间地址"
            />
            <el-button type="primary" class="startListen" @click="startListen">
                开始采集
            </el-button>

            <el-button type="success" class="startListen" @click="openLogin">
                登录抖音
            </el-button>

            <el-button type="primary" class="startListen" @click="openWindow">
                新窗口
            </el-button>
        </div>
        <!-- 下面直播间:左侧直播，右侧评论 -->
        <div class="liveBox">
            <!-- 视频播放容器 -->
            <div class="liveVideo">
                <!-- 主播头像信息：固定位置 -->
                <div class="ownerBox">
                    <!-- 头像 -->
                    <img :src="liveInfo.avatar" alt="头像" class="avatar" />
                    <div class="nickBox">
                        <span class="nickName">{{ liveInfo.name }}</span>
                        <span class="fans">
                            {{ liveInfo.totalLike }}本场点赞
                        </span>
                    </div>
                </div>
                <!-- 右侧本场点赞等信息 -->
                <div class="likeInfo">
                    <div class="fans" @click="handlePay">
                        主播粉丝：{{ liveInfo.fans }}
                    </div>
                    <div class="customer">
                        在线观众：{{ liveInfo.customer }}
                    </div>
                    <div class="diamond">主播收益：{{ diamond }} 音浪 (到手 ¥{{ diamondRMB }})</div>
                </div>
                <!-- 视频播放器 -->
                <div id="dplayer" class="dplayer"></div>
                <!-- 直播结束 -->
                <div v-if="liveInfo.status === 4" class="over">直播已结束</div>
            </div>
            <!-- 长列表优化 -->
            <DynamicScroller
                :items="messageList"
                :min-item-size="32"
                class="liveMeg"
                id="liveMsg"
                ref="liveMsg"
                v-if="messageList.length"
            >
                <template v-slot="{ item, active }">
                    <DynamicScrollerItem
                        :item="item"
                        :active="active"
                        class="msgBox"
                        :size-dependencies="[item.name, item.msg]"
                        :data-index="item.id"
                    >
                        <div class="content">
                            <span class="name">{{ item.name }}：</span>
                            <span class="msg">{{ item.msg }}</span>
                        </div>
                    </DynamicScrollerItem>
                </template>
            </DynamicScroller>
        </div>
        <!-- 设置推流地址 -->
        <el-icon :size="20" class="pushUrl" @click="dialogVisible = true">
            <Setting />
        </el-icon>
    </div>
    <!-- 设置推流地址 -->
    <el-dialog
        v-model="dialogVisible"
        title="设置推送地址"
        center
        :show-close="false"
        width="540"
    >
        <div class="setBox">
            <el-input v-model="pushUrl" placeholder="请输入推送地址" />
            <!-- 选择消息类型 -->
            <div class="messageSel">
                <span>选择消息类型：</span>
                <el-checkbox-group v-model="checkList">
                    <el-checkbox label="聊天" value="chat" />
                    <el-checkbox label="礼物" value="gift" />
                    <el-checkbox label="点赞" value="like" />
                    <el-checkbox label="关注" value="follow" />
                    <el-checkbox label="进来" value="comein" />
                </el-checkbox-group>
            </div>
            <!-- 添加录制视频和弹幕 -->
            <div class="messageSel">
                <span>直播录制配置：</span>
                <el-checkbox-group v-model="recordVideo">
                    <el-checkbox label="开启录制" value="open" />
                    <el-checkbox label="录制弹幕" value="chat" />
                    <el-checkbox label="录制礼物" value="gift" />
                </el-checkbox-group>
            </div>
            <div class="tips">
                *推送的消息会以POST请求的形式发送到该地址，请确保该地址能够接收POST请求
            </div>
        </div>
        <template #footer>
            <div class="dialog-footer">
                <el-button @click="dialogVisible = false">取消</el-button>
                <el-button type="primary" @click="dialogVisible = false">
                    确定
                </el-button>
            </div>
        </template>
    </el-dialog>
</template>

<script setup lang="ts">
import { Setting } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ref, computed } from 'vue'
import { DPlayerImp, LiveInfoImp } from '@/types'
import Logo from '@/assets/logo.png'
import { ConnectionConfig } from 'tauri-plugin-websocket-api'
import { douyin } from '@/proto/dy.js'
import { ElMessage } from 'element-plus'
import DPlayer from 'dplayer'
import Hls from 'hls.js'
import Flv from 'flv.js'
import pako from 'pako'
import SocketCli from '@/utils/RustSocket'
import { emit, listen } from '@tauri-apps/api/event'

// 直播间地址
const inputUrl = ref(localStorage.getItem('url') || '')
const dialogVisible = ref(false)
const messageList = ref([
    {
        id: '1',
        name: '1024小神',
        msg: '欢迎使用直播盒子，输入直播地址开始安静看直播，没有刷礼物功能，所以理性看播，不要乱消费',
    },
])
// websocket client
let socketClient: SocketCli

// 主播信息
const liveInfo = ref({
    uid: '888888',
    status: 0, // 直播间状态4是已结束
    title: '直播标题',
    name: 'Livebox',
    roomId: '888888',
    avatar: Logo,
    fans: 0,
    customer: 0,
    totalLike: 0,
    signature: '',
})

// 主播收益（音浪）
const diamond = ref(0)

// 计算主播实际到手金额（10 音浪 = 1 元，平台抽成 50%）
const diamondRMB = computed(() => {
    const totalRMB = diamond.value / 10  // 礼物总价值
    const actualIncome = totalRMB * 0.5  // 主播实际到手（50%）
    return actualIncome.toFixed(2)
})

// 推送流地址（默认地址）
const pushUrl = ref('http://localhost:5001/webhook')
// 选中消息类型（默认只推送聊天消息）
const checkList = ref<string[]>(['chat'])
// 录制视频
const recordVideo = ref<string[]>([])

// 聊天消息盒子
const liveMsg = ref()

// 直播播放器
let dplayer: DPlayerImp | null = null
let liveNum = 100

// 打开登录窗口
const openLogin = async () => {
    try {
        const result = await invoke('open_login_page')
        ElMessage.success('登录窗口已打开，请在浏览器中登录抖音，登录后 Cookie 会自动保存')
        console.log('✅ 登录窗口:', result)
    } catch (error) {
        ElMessage.error('打开登录窗口失败: ' + error)
        console.error('❌ 打开登录窗口失败:', error)
    }
}

// 新窗口
const openWindow = () => {
    invoke('open_window', {
        appUrl: inputUrl.value,
        appName: '直播盒子' + liveNum++,
        platform: 'web',
        userAgent: navigator.userAgent,
        resize: false,
        width: 1000,
        height: 800,
        jsContent: '',
    })
}

// call pay
const handlePay = () => {
    console.log('emit handlepay')
    emit('handlepay')
}

// listen('handlepay', () => {
//     console.log('Received handlepay:')
// })

// 开始监听
const startListen = async () => {
    const url = inputUrl.value.trim()
    // console.log('直播间地址:', proto)
    localStorage.setItem('url', url)
    // 先清空历史直播
    clearLivex()
    // 再开始新的直播
    if (url.trim()) {
        // 根据直播间地址获取roomid等字段
        const roomJson: LiveInfoImp = await invoke('get_live_html', { url })
        // console.log('获取到的直播房间信息:', roomJson)
        // roomInfo
        const roomInfo = JSON.parse(roomJson.room_info)
        console.log('roomInfo----', roomInfo)
        // 获取主播的头像昵称粉丝数等信息
        if (roomInfo.id_str) {
            // 如果状态是4就是停播，只有直播的信息
            if (roomInfo.status) {
                ElMessage.success('open live success!')
                liveInfo.value = {
                    uid: roomInfo.owner.id_str,
                    status: roomInfo.status,
                    title: roomInfo.title,
                    name: roomInfo.owner.nickname,
                    roomId: roomInfo.id_str,
                    avatar: roomInfo.owner.avatar_thumb.url_list[0],
                    fans: 0,
                    customer: roomInfo.user_count_str,
                    totalLike: roomInfo.stats.total_user_str,
                    signature: 'roomInfo.signature',
                }
                // 加载直播视频:可能没有HD1
                let videoUrl = roomInfo.stream_url.flv_pull_url[
                    roomInfo.stream_url.default_resolution
                ].replace('http://', 'https://')
                loadLive(videoUrl)
                // 加载websocket
                creatSokcet(roomInfo.id_str, roomJson.unique_id, roomJson.ttwid)
            } else {
                ElMessage.success('live is over!')
                liveInfo.value = {
                    uid: roomInfo.id_str,
                    status: 4,
                    title: '已停播',
                    name: roomInfo.nickname,
                    roomId: roomInfo.id_str,
                    avatar: roomInfo.avatar_thumb.url_list[0],
                    fans: 0,
                    customer: 0,
                    totalLike: 0,
                    signature: 'roomInfo.signature',
                }
                // 清空播放器
                destroyPlayer()
            }
        } else {
            console.log('没有获取到')
            ElMessage.error('open live error')
        }
    }
}

// 清空直播和聊天内容
const clearLivex = () => {
    // console.log('清空')
    dplayer?.destroy()
    messageList.value = [
        {
            id: '1',
            name: '1024小神',
            msg: '欢迎使用直播盒子，输入直播地址开始安静看直播，没有刷礼物功能，所以理性看播，不要乱消费',
        },
    ]
    socketClient?.disconnect()
}

// 创建websokcet
const creatSokcet = async (roomId: string, uniqueId: string, ttwid: string) => {
    console.log('🔌 [WebSocket] 开始创建 WebSocket 连接...')
    console.log('  roomId:', roomId)
    console.log('  uniqueId:', uniqueId)
    console.log('  ttwid:', ttwid ? (ttwid.substring(0, 20) + '...') : '(空)')

    let sign = window.creatSignature(roomId, uniqueId)
    console.log('  signature:', sign ? '已生成' : '生成失败')
    // 组装参数
    let socketUrl = `wss://webcast5-ws-web-lf.douyin.com/webcast/im/push/v2/?room_id=${roomId}&compress=gzip&version_code=180800&webcast_sdk_version=1.0.14-beta.0&live_id=1&did_rule=3&user_unique_id=${uniqueId}&identity=audience&signature=${sign}&aid=6383&device_platform=web&browser_language=zh-CN&browser_platform=Win32&browser_name=Mozilla&browser_version=5.0+%28Windows+NT+10.0%3B+Win64%3B+x64%29+AppleWebKit%2F537.36+%28KHTML%2C+like+Gecko%29+Chrome%2F126.0.0.0+Safari%2F537.36+Edg%2F126.0.0.0`
    // header - 如果 ttwid 为空，就不发送 cookie（游客模式）
    const options: ConnectionConfig = {
        writeBufferSize: 20000,
        headers: ttwid ? {
            cookie: 'ttwid=' + ttwid,
            'user-agent':
                'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0',
        } : {
            'user-agent':
                'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0',
        },
    }

    console.log('🔌 [WebSocket] 连接模式:', ttwid ? '使用 ttwid Cookie' : '游客模式（无 Cookie）')
    // ping消息
    const pingMsg = douyin.PushFrame.encode({ payloadType: 'hb' }).finish()
    // webscoket
    socketClient = new SocketCli(socketUrl, options, onMessage, pingMsg)
}
// 加载直播视频
const loadLive = (videoUrl: string, live: boolean = true) => {
    // 根据不同的视频加载不同的播放器
    if (videoUrl.includes('m3u8')) {
        dplayer = new DPlayer({
            container: document.getElementById(`dplayer`),
            screenshot: false,
            autoplay: true,
            live: live,
            lang: 'zh-cn', // zh-cn // en
            video: {
                url: '',
                type: 'customHls',
                customType: {
                    customHls: function (video: any, _: any) {
                        const hls = new Hls() //实例化Hls  用于解析m3u8
                        hls.loadSource(videoUrl)
                        hls.attachMedia(video)
                    },
                },
            },
        })
    } else if (videoUrl.includes('mp4')) {
        dplayer = new DPlayer({
            container: document.getElementById(`dplayer`),
            live: live,
            autoplay: true,
            screenshot: false,
            fullScreen: false,
            lang: 'zh-cn', // zh-cn // en
            video: {
                url: videoUrl,
                type: 'mp4',
            },
        })
    } else if (videoUrl.includes('flv')) {
        dplayer = new DPlayer({
            container: document.getElementById(`dplayer`),
            screenshot: false,
            live: live,
            autoplay: true,
            lang: 'zh-cn', // zh-cn // en
            video: {
                url: videoUrl,
                type: 'customFlv',
                customType: {
                    customFlv: function (video: any, _: any) {
                        const flvPlayer = Flv.createPlayer({
                            type: 'flv',
                            url: videoUrl,
                        })
                        flvPlayer.attachMediaElement(video)
                        flvPlayer.load()
                    },
                },
            },
        })
    }
    // 立即播放视频
    // dplayer?.play()
}

// 销毁播放器
const destroyPlayer = () => {
    if (dplayer) {
        dplayer.destroy()
        dplayer = null
    }
}

// 消息列表添加消息：长列表优化
const pushMsg = (msg: any) => {
    // 列表长度限制在50个
    messageList.value.push(msg)
}

// 收到websocket消息回调
const onMessage = (msg: any) => {
    console.log('📨 [WebSocket] 收到消息，数据长度:', msg.data?.length || 0)
    // 解析消息
    const decodeMsg = douyin.PushFrame.decode(msg.data)
    console.log('📨 [WebSocket] 消息类型:', decodeMsg.payloadType)
    // 滚动盒子到底部
    if (liveMsg.value) {
        const msgDom: HTMLElement | null = document.getElementById('liveMsg')
        if (msgDom) {
            msgDom.scrollTop = msgDom.scrollHeight
        }
    }
    // 解压缩应该是没问题，
    const gzipData = pako.inflate(decodeMsg.payload)
    // console.log('gzipData--', gzipData)
    // Response解码，有问题, 所以要用Response.decode解码也应该是数字类型
    const response = douyin.Response.decode(gzipData)
    // 遍历 payloadPackage.messagesList
    // 判断是否需要回复，自动回复
    if (response.needAck) {
        const ack = douyin.PushFrame.encode({
            payloadType: 'ack',
            logId: decodeMsg.logId,
        }).finish()
        socketClient?.send(ack)
    }
    // 解析直播消息
    handleMessage(response.messagesList)
}

// 遍历消息数组，拿到具体的消息
const handleMessage = (messageList: douyin.Message) => {
    console.log('📨 [WebSocket] 消息列表长度:', messageList.length)
    messageList.forEach((msg) => {
        console.log('📨 [WebSocket] 消息方法:', msg.method)
        // 判断消息类型
        switch (msg.method) {
            // 反对分数
            case 'WebcastMatchAgainstScoreMessage':
                // console.log('反对分数')
                break
            // 点赞数
            case 'WebcastLikeMessage':
                // console.log('点赞数')
                likeLive(msg.payload)
                break
            // 成员进入直播间消息
            case 'WebcastMemberMessage':
                // console.log('成员进入直播间消息')
                enterLive(msg.payload)
                break
            // 礼物消息
            case 'WebcastGiftMessage':
                // console.log('礼物消息')
                decodeGift(msg.payload)
                break
            // 聊天弹幕消息
            case 'WebcastChatMessage':
                // console.log('聊天弹幕消息')
                decodeChat(msg.payload)
                break
            // 关注消息
            case 'WebcastSocialMessage':
                // console.log('联谊会消息')
                followLive(msg.payload)
                break
            // 更新粉丝票
            case 'WebcastUpdateFanTicketMessage':
                // console.log('更新粉丝票')
                break
            // 公共文本消息
            case 'WebcastCommonTextMessage':
                // console.log('公共文本消息')
                break
            // 商品改变消息
            case 'WebcastProductChangeMessage':
                // console.log('商品改变消息')
                break
            // 直播间统计消息
            case 'WebcastRoomUserSeqMessage':
                // console.log('直播间统计消息')
                countLive(msg.payload)
                break
            // 待解析方法
            default:
                console.log('待解析方法' + msg.method)
                break
        }
    })
}
// 解析弹幕消息
const decodeChat = (data) => {
    // 校验消息
    const chatMsg = douyin.ChatMessage.decode(data)
    console.log('💬 [聊天消息] 用户:', chatMsg.user?.nickName, '内容:', chatMsg.content)
    const { common, user, content } = chatMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: content,
    }
    if (checkList.value.includes('chat')) {
        messageList.value.push(message)
        console.log('💬 [聊天消息] 已添加到消息列表，当前列表长度:', messageList.value.length)
    } else {
        console.log('💬 [聊天消息] 聊天类型未勾选，不显示消息')
    }

    // 推送到配置的 URL
    if (pushUrl.value && checkList.value.includes('chat')) {
        pushMessageToUrl('chat', message, chatMsg)
    }
}
// 解析礼物消息
const decodeGift = (data) => {
    const giftMsg = douyin.GiftMessage.decode(data)
    // console.log('giftMsg---', giftMsg)
    const { common, user, gift, repeatCount } = giftMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: `送出${gift.name} x${repeatCount}个`,
    }
    checkList.value.includes('gift') && messageList.value.push(message)
    // 计算主播收益
    diamond.value = diamond.value + gift.diamondCount * repeatCount

    // 推送到配置的 URL
    if (pushUrl.value && checkList.value.includes('gift')) {
        pushMessageToUrl('gift', message, giftMsg)
    }
}

// 进入房间
const enterLive = (data) => {
    const enteryMsg = douyin.MemberMessage.decode(data)
    const { common, user } = enteryMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: '来了',
    }
    checkList.value.includes('comein') && messageList.value.push(message)

    // 推送到配置的 URL
    if (pushUrl.value && checkList.value.includes('comein')) {
        pushMessageToUrl('comein', message, enteryMsg)
    }
}

// 点赞消息
const likeLive = (data) => {
    const likeMsg = douyin.LikeMessage.decode(data)
    // console.log('likeMsg---', likeMsg)
    const { common, user, total } = likeMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: `为主播点赞了`,
    }
    liveInfo.value = {
        ...liveInfo.value,
        totalLike: total,
    }
    checkList.value.includes('like') && messageList.value.push(message)

    // 推送到配置的 URL
    if (pushUrl.value && checkList.value.includes('like')) {
        pushMessageToUrl('like', message, likeMsg)
    }
}

// 关注主播
const followLive = (data) => {
    const followMsg = douyin.SocialMessage.decode(data)
    const { common, user, followCount } = followMsg
    const message = {
        id: common.msgId,
        name: user.nickName,
        msg: `关注了主播`,
    }
    liveInfo.value = {
        ...liveInfo.value,
        fans: followCount,
    }
    checkList.value.includes('follow') && messageList.value.push(message)

    // 推送到配置的 URL
    if (pushUrl.value && checkList.value.includes('follow')) {
        pushMessageToUrl('follow', message, followMsg)
    }
}

// 推送消息到配置的 URL
const pushMessageToUrl = async (type, message, rawData) => {
    if (!pushUrl.value) return

    try {
        const payload = {
            type: type,
            data: message,
            raw: rawData,
            timestamp: Date.now(),
            room_id: liveInfo.value.roomId,
        }

        const response = await fetch(pushUrl.value, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(payload),
        })

        if (!response.ok) {
            console.error('推送失败:', response.status, response.statusText)
        }
    } catch (error) {
        console.error('推送消息到URL失败:', error)
    }
}

// 直播间统计
const countLive = (data) => {
    const countMsg = douyin.RoomUserSeqMessage.decode(data)
    // console.log('countLive---', countMsg)
    liveInfo.value = {
        ...liveInfo.value,
        customer: countMsg.onlineUserForAnchor,
    }
}

// 弹幕消息列表：优化
var lastScrollTop = 0
const msgScroll = (event) => {
    console.log('列表滚动', event)
    const { scrollTop } = event.target
    if (scrollTop < lastScrollTop) {
        // 向上滚动
        console.log('向上滚动')
    } else if (scrollTop > lastScrollTop) {
        // 向下滚动
        console.log('向下滚动')
    }
    lastScrollTop = scrollTop
}
</script>

<style scoped lang="scss">
.container {
    width: 100%;
    height: 100%;
    padding: 20px 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    background-color: #f5f5f5;
    .liveUrl {
        display: flex;
        flex-direction: row;
        justify-content: center;
        align-items: center;
        height: 36px;
        width: 100%;

        .urlInput {
            width: 50%;
            height: 36px;
            border-radius: 10px;
            padding-left: 16px;
            font-size: 15px;
            letter-spacing: 0.15px;
            border: none;
            outline: none;
            color: black;
            font-family: 'Montserrat', sans-serif;
            background-color: #ecf0f3;
            transition: 0.25s ease;
            box-shadow: inset 2px 2px 4px #d1d9e6, inset -2px -2px 4px #d1d9e6;

            &:focus {
                box-shadow: inset 4px 4px 4px #d1d9e6,
                    inset -4px -4px 4px #e1e5ec;
            }
        }

        .startListen {
            margin-left: 16px;
            box-shadow: 0 0 6px 2px #bfc7d4;
        }
    }

    .liveBox {
        flex: 1;
        display: flex;
        width: 100%;
        height: 90%;
        padding: 20px 20px 0 20px;
        flex-direction: row;
        justify-content: center;
        .liveVideo {
            position: relative;
            width: 72%;
            height: 100%;
            border-radius: 10px;
            background-image: url('@/assets/images/liveBg.webp');
            background-position: center;
            background-size: cover;
            background-repeat: no-repeat;
            background-color: rgba(0, 0, 0, 0.5);
            box-shadow: 0 0 10px 2px gray;
            display: flex;
            flex-direction: row;
            justify-content: center;
            align-items: center;

            .ownerBox {
                position: absolute;
                top: 10px;
                left: 10px;
                height: 40px;
                display: flex;
                flex-direction: row;
                align-items: center;
                background-color: #0000008b;
                padding: 10px 4px;
                border-radius: 20px;
                z-index: 999;
                user-select: none;

                .avatar {
                    width: 32px;
                    height: 32px;
                    border-radius: 50%;
                    margin-right: 5px;
                }

                .nickBox {
                    display: flex;
                    flex-direction: column;
                    justify-content: center;
                    align-items: flex-start;
                    margin-right: 10px;

                    .nickName {
                        font-size: 14px;
                        color: white;
                        user-select: none;
                    }

                    .fans {
                        font-size: 11px;
                        color: #ccc;
                        user-select: none;
                    }
                }
            }

            .likeInfo {
                position: absolute;
                top: 10px;
                right: 10px;
                height: 40px;
                display: flex;
                flex-direction: row;
                align-items: center;
                // background-color: #0000008b;
                padding: 10px 4px;
                border-radius: 20px;
                z-index: 999;
                user-select: none;
                color: white;

                .customer {
                    margin: 0 20px;
                }
            }

            .dplayer {
                width: 100%;
                height: 100%;
                border-radius: 10px;
            }

            .over {
                position: absolute;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                display: flex;
                flex-direction: row;
                justify-content: center;
                align-items: center;
                color: white;
                font-size: 25px;
                font-weight: bold;
                user-select: none;
                text-shadow: 0 0 6px 2px black;
            }
        }

        .liveMeg {
            flex: 1;
            margin-left: 10px;
            background-color: #252632;
            border-radius: 10px;
            box-shadow: 0 0 10px 2px gray;
            scrollbar-color: #363741 transparent;
            scrollbar-width: thin;
            overflow-y: scroll;

            .msgBox {
                display: flex;
                flex-direction: row;
                padding: 5px;
                white-space: wrap;

                .name {
                    color: #8ce7ff;
                    margin-right: 2px;
                    white-space: nowrap;
                }

                .msg {
                    color: white;
                    white-space: wrap;
                }
            }
        }
    }

    .pushUrl {
        position: fixed;
        top: 3vh;
        right: 3vh;
    }
}

.setBox {
    margin: 2vh 20px;

    .messageSel {
        margin-top: 4px;
    }

    .tips {
        font-size: small;
        color: #999;
    }
}
</style>
