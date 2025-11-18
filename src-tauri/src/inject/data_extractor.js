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

        // 方法1: 从 window 对象中查找
        if (window.__INITIAL_STATE__ || window.ROOM_DATA || window.__INITIAL_PROPS__) {
            console.log('📦 从 window 对象提取数据...');

            // 尝试从不同的全局变量中提取
            const stateData = window.__INITIAL_STATE__ || window.ROOM_DATA || window.__INITIAL_PROPS__;
            console.log('找到状态数据:', Object.keys(stateData));

            // 深度搜索数据结构（扩展搜索更多字段名）
            const searchKeys = [
                'title', 'nickname', 'room_title', 'roomTitle',  // 标题相关
                'user_unique_id', 'userId', 'user_id', 'roomId', 'room_id', 'web_rid',  // ID 相关
                'stream_url', 'pull_url', 'streamUrl', 'flv_pull_url', 'hls_pull_url'  // 推流地址相关
            ];
            const searchResult = deepSearch(stateData, searchKeys);
            console.log('深度搜索结果:', searchResult);
            console.log('完整数据对象键:', Object.keys(stateData));

            // 提取标题
            data.title = searchResult.title || searchResult.nickname || searchResult.room_title || searchResult.roomTitle || '';

            // 提取主播ID
            data.user_unique_id = searchResult.user_unique_id || searchResult.userId || searchResult.user_id ||
                                 searchResult.roomId || searchResult.room_id || searchResult.web_rid || '';

            // 提取推流地址
            data.stream_url = searchResult.stream_url || searchResult.pull_url || searchResult.streamUrl ||
                             searchResult.flv_pull_url || searchResult.hls_pull_url || '';

            data.room_store = JSON.stringify(stateData);
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
