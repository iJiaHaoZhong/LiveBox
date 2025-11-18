// 直播间数据提取脚本
// 在浏览器窗口中运行，直接从页面提取直播间信息

(function() {
    console.log('🔧 直播间数据提取脚本已加载');

    let dataExtracted = false;
    let checkCount = 0;
    const MAX_CHECKS = 60; // 最多检查 60 秒

    // 提取直播间数据的函数
    function extractLiveRoomData() {
        checkCount++;

        if (checkCount > MAX_CHECKS) {
            console.log('⏱ 超时：未能提取数据');
            sendError('timeout', '提取数据超时（60秒）');
            clearInterval(extractInterval);
            return;
        }

        // 检查页面是否加载完成
        const currentUrl = window.location.href;
        const pageTitle = document.title || '';
        const pageHtml = document.body ? document.body.innerHTML : '';
        const pageHtmlLength = pageHtml.length;

        // 确保页面已加载
        if (pageHtmlLength < 1000) {
            if (checkCount % 5 === 0) {
                console.log(`⏳ 等待页面加载... (${checkCount}秒)`);
            }
            return;
        }

        // 检查是否在验证码页面
        const isOnCaptchaPage = pageTitle.includes('验证码') ||
                               pageHtml.includes('验证码中间页') ||
                               pageHtml.includes('middle_page_loading') ||
                               pageHtml.includes('TTGCaptcha');

        if (isOnCaptchaPage) {
            if (checkCount % 5 === 0) {
                console.log(`⏳ 等待验证码验证... (${checkCount}秒)`);
            }
            return;
        }

        // 检查是否成功加载直播间页面
        const hasLiveRoomContent = pageHtml.includes('live_room') ||
                                  pageHtml.includes('room_data') ||
                                  pageHtml.includes('webcast');

        if (!hasLiveRoomContent) {
            if (checkCount % 5 === 0) {
                console.log(`⏳ 等待直播间页面加载... (${checkCount}秒)`);
            }
            return;
        }

        // 页面已加载完成，开始提取数据
        if (!dataExtracted) {
            console.log('✅ 直播间页面已加载，开始提取数据...');

            try {
                // 尝试从多个来源提取数据
                const data = extractFromPage();

                // 输出详细的提取结果
                console.log('🔍 提取结果:');
                console.log('  - 标题:', data.title || '(空)');
                console.log('  - 主播ID:', data.user_unique_id || '(空)');
                console.log('  - 推流地址:', data.stream_url || '(空)');
                console.log('  - room_store 长度:', data.room_store.length);

                if (data && data.title) {
                    dataExtracted = true;
                    console.log('✅ 成功提取直播间数据！');
                    console.log('📝 标题:', data.title);
                    console.log('🎬 主播ID:', data.user_unique_id || '(未找到)');
                    console.log('🔗 推流地址:', data.stream_url ? '已找到' : '未找到');

                    sendData(data);
                    clearInterval(extractInterval);
                } else {
                    console.log('⚠️ 提取的数据不完整（标题为空），继续尝试...');
                }
            } catch (error) {
                console.error('❌ 提取数据时出错:', error);
                sendError('extract_failed', error.message);
                clearInterval(extractInterval);
            }
        }
    }

    // 从页面提取数据
    function extractFromPage() {
        const data = {
            title: '',
            user_unique_id: '',
            stream_url: '',
            room_store: ''
        };

        // 方法1: 从 window.__STORE__ 对象中查找（抖音实际使用的数据结构）
        if (window.__STORE__) {
            console.log('📦 从 window.__STORE__ 提取数据...');
            console.log('找到 STORE，包含键:', Object.keys(window.__STORE__));

            try {
                const store = window.__STORE__;

                // 从 roomStore 提取直播间信息
                if (store.roomStore && store.roomStore.roomInfo) {
                    console.log('✓ 找到 roomStore');
                    const roomInfo = store.roomStore.roomInfo;

                    // 正确的字段路径：roomInfo.room.title
                    data.title = roomInfo.room?.title || '';

                    // 提取房间ID
                    const roomId = roomInfo.roomId || roomInfo.web_rid || '';

                    console.log('  roomStore 标题:', data.title || '(未找到)');
                    console.log('  roomStore 房间ID:', roomId || '(未找到)');
                }

                // 从 userStore 提取用户信息
                if (store.userStore && store.userStore.userInfo) {
                    console.log('✓ 找到 userStore');
                    const userInfo = store.userStore.userInfo;

                    // 正确的字段路径：userInfo.display_id 或 userInfo.id_str
                    data.user_unique_id = userInfo.display_id || userInfo.id_str || userInfo.web_rid || '';

                    console.log('  userStore 用户ID:', data.user_unique_id || '(未找到)');
                }

                // 从 streamStore 提取推流信息
                if (store.streamStore && store.streamStore.streamData) {
                    console.log('✓ 找到 streamStore');
                    const streamData = store.streamStore.streamData;

                    // 正确的字段路径：streamData.H264_streamData
                    // 尝试提取推流地址（从 H264 或 H265）
                    const h264Data = streamData.H264_streamData;
                    const h265Data = streamData.H265_streamData;

                    // 尝试从 streamData 中提取 URL
                    data.stream_url = h264Data?.main?.flv ||
                                     h264Data?.main?.hls ||
                                     h265Data?.main?.flv ||
                                     h265Data?.main?.hls || '';

                    console.log('  streamStore 推流地址:', data.stream_url ? '已找到' : '(未找到)');
                }

                // 将整个 STORE 序列化存储（使用 JSON.stringify 处理 MobX 对象）
                try {
                    // MobX 对象需要转换为普通对象
                    const storeData = {
                        roomStore: toPlainObject(store.roomStore),
                        userStore: toPlainObject(store.userStore),
                        streamStore: toPlainObject(store.streamStore),
                    };
                    data.room_store = JSON.stringify(storeData);
                    console.log('✓ 序列化 store 数据，长度:', data.room_store.length);
                } catch (e) {
                    console.warn('⚠️  序列化 store 失败:', e.message);
                    // 备用方案：只存储基本信息
                    data.room_store = JSON.stringify({
                        title: data.title,
                        user_unique_id: data.user_unique_id,
                        stream_url: data.stream_url
                    });
                }

            } catch (error) {
                console.error('❌ 从 __STORE__ 提取数据时出错:', error);
            }
        }

        // 方法2: 从页面 HTML 中的 script 标签提取
        if (!data.title) {
            console.log('📄 从 HTML script 标签提取数据...');
            const scripts = document.querySelectorAll('script');

            for (let script of scripts) {
                const content = script.textContent || script.innerHTML;

                // 查找包含 ROOM 或 INITIAL 的数据
                if (content.includes('ROOM') || content.includes('INITIAL') || content.includes('roomStore')) {
                    try {
                        // 尝试提取 JSON 数据
                        const jsonMatch = content.match(/\{[\s\S]*"title"[\s\S]*\}/);
                        if (jsonMatch) {
                            const jsonData = JSON.parse(jsonMatch[0]);
                            const searchResult = deepSearch(jsonData, ['title', 'nickname', 'user_unique_id']);

                            data.title = data.title || searchResult.title || searchResult.nickname || '';
                            data.user_unique_id = data.user_unique_id || searchResult.user_unique_id || '';

                            if (data.title) {
                                console.log('✓ 从 script 标签中找到数据');
                                break;
                            }
                        }
                    } catch (e) {
                        // 忽略解析错误，继续查找
                    }
                }
            }
        }

        // 方法3: 从页面元素中提取
        if (!data.title) {
            console.log('🏷️ 从页面元素提取数据...');

            // 尝试从 meta 标签获取标题
            const titleMeta = document.querySelector('meta[property="og:title"]') ||
                             document.querySelector('meta[name="title"]');
            if (titleMeta) {
                data.title = titleMeta.getAttribute('content') || '';
            }

            // 如果还是没有，使用 document.title
            if (!data.title) {
                data.title = document.title.replace(/[-_].*$/, '').trim();
            }
        }

        return data;
    }

    // 深度搜索对象中的键
    function deepSearch(obj, keys, maxDepth = 10, currentDepth = 0) {
        const result = {};

        if (currentDepth > maxDepth || !obj || typeof obj !== 'object') {
            return result;
        }

        for (let key in obj) {
            if (keys.includes(key)) {
                result[key] = obj[key];
            }

            if (typeof obj[key] === 'object' && obj[key] !== null) {
                const childResult = deepSearch(obj[key], keys, maxDepth, currentDepth + 1);
                Object.assign(result, childResult);
            }
        }

        return result;
    }

    // 将 MobX observable 对象转换为普通对象
    function toPlainObject(obj, maxDepth = 5, currentDepth = 0) {
        if (currentDepth > maxDepth || obj === null || obj === undefined) {
            return obj;
        }

        // 基本类型直接返回
        if (typeof obj !== 'object') {
            return obj;
        }

        // 数组
        if (Array.isArray(obj)) {
            return obj.map(item => toPlainObject(item, maxDepth, currentDepth + 1));
        }

        // 对象
        const plainObj = {};
        for (let key in obj) {
            // 跳过 MobX 内部属性和函数
            if (key.startsWith('$') || key.startsWith('_') || typeof obj[key] === 'function') {
                continue;
            }

            try {
                const value = obj[key];
                plainObj[key] = toPlainObject(value, maxDepth, currentDepth + 1);
            } catch (e) {
                // 忽略无法访问的属性
            }
        }

        return plainObj;
    }

    // 发送数据给后端
    function sendData(data) {
        try {
            // URL 编码数据
            const encodedData = encodeURIComponent(JSON.stringify(data));
            window.location.hash = '__LIVE_DATA__=' + encodedData;

            console.log('✅ 数据已准备好，正在传递给后端...');
            console.log('📝 URL hash 已设置: #__LIVE_DATA__=[数据]');

            showSuccessMessage();
        } catch (error) {
            console.error('❌ 发送数据失败:', error);
            sendError('send_failed', error.message);
        }
    }

    // 发送错误信息
    function sendError(errorType, errorMessage) {
        const errorData = {
            error: errorType,
            message: errorMessage
        };
        const encodedData = encodeURIComponent(JSON.stringify(errorData));
        window.location.hash = '__LIVE_ERROR__=' + encodedData;
        console.log('❌ 错误已传递给后端');
    }

    // 显示成功消息
    function showSuccessMessage() {
        const messageDiv = document.createElement('div');
        messageDiv.innerHTML = `
            <div style="
                position: fixed;
                top: 50%;
                left: 50%;
                transform: translate(-50%, -50%);
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                padding: 30px 50px;
                border-radius: 15px;
                font-size: 18px;
                font-weight: bold;
                box-shadow: 0 10px 40px rgba(0,0,0,0.3);
                z-index: 999999;
                text-align: center;
                animation: fadeIn 0.3s ease-in;
            ">
                <div style="font-size: 48px; margin-bottom: 15px;">✅</div>
                <div>数据提取成功！</div>
                <div style="font-size: 14px; margin-top: 10px; opacity: 0.9;">窗口将自动关闭...</div>
            </div>
        `;

        const style = document.createElement('style');
        style.textContent = `
            @keyframes fadeIn {
                from { opacity: 0; transform: translate(-50%, -60%); }
                to { opacity: 1; transform: translate(-50%, -50%); }
            }
        `;
        document.head.appendChild(style);
        document.body.appendChild(messageDiv);
    }

    console.log('🚀 开始监听页面数据...');

    // 每秒检查一次
    const extractInterval = setInterval(extractLiveRoomData, 1000);

    // 立即执行一次
    extractLiveRoomData();
})();
