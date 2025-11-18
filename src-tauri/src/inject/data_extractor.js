(function() {
    'use strict';

    console.log('🔧 直播间数据提取脚本已加载');

    // 将 MobX Proxy 对象转换为普通对象
    function toPlainObject(obj, maxDepth = 5, currentDepth = 0) {
        if (currentDepth > maxDepth || obj === null || obj === undefined) {
            return obj;
        }

        if (typeof obj !== 'object') {
            return obj;
        }

        if (Array.isArray(obj)) {
            return obj.map(item => toPlainObject(item, maxDepth, currentDepth + 1));
        }

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

    // 等待页面加载和数据就绪
    let checkCount = 0;
    const maxChecks = 30; // 最多检查 30 次（15秒）
    const checkInterval = 500; // 每 500ms 检查一次

    const intervalId = setInterval(() => {
        checkCount++;

        // 检查文档是否加载完成
        if (document.readyState !== 'complete' && checkCount < maxChecks) {
            console.log(`⏳ 等待页面加载... (${checkCount * 0.5}秒)`);
            return;
        }

        // 检查是否有 __STORE__ 对象
        if (!window.__STORE__) {
            if (checkCount < maxChecks) {
                console.log(`⏳ 等待 __STORE__ 就绪... (${checkCount * 0.5}秒)`);
                return;
            } else {
                console.log('❌ 超时：未找到 window.__STORE__ 对象');
                clearInterval(intervalId);
                return;
            }
        }

        // 数据提取逻辑
        console.log('✅ 直播间页面已加载，开始提取数据...');
        clearInterval(intervalId);

        try {
            extractFromPage();
        } catch (e) {
            console.log('❌ 数据提取出错:', e);
        }
    }, checkInterval);

    function extractFromPage() {
        console.log('🔍 开始从页面提取直播间数据...');

        // 初始化数据结构（用于传递给后端）
        let data = {
            title: '',           // 直播间标题（用于显示）
            user_unique_id: '', // 访问者的唯一ID（用于生成 WebSocket 签名）
            ttwid: '',          // Cookie 中的 ttwid（用于 WebSocket 认证）
            room_store: ''      // 直播间完整信息的 JSON 字符串（包含 room_id 等）
        };

        // 从 window.__STORE__ 对象提取数据（抖音实际使用的数据结构）
        if (window.__STORE__) {
            console.log('📦 从 window.__STORE__ 提取数据...');
            console.log('找到 STORE，包含键:', Object.keys(window.__STORE__));

            try {
                const store = window.__STORE__;

                // 1. 从 roomStore 提取直播间信息
                if (store.roomStore && store.roomStore.roomInfo) {
                    console.log('✓ 找到 roomStore.roomInfo');
                    const roomInfo = store.roomStore.roomInfo;

                    // 提取标题
                    data.title = roomInfo.room?.title || roomInfo.title || '';

                    // ⚠️ 重要：前端期望的是 roomInfo.room 对象，而不是整个 roomInfo
                    // 原来的 HTTP 方式提取的正则：roomInfo\\":{\\"room\\":(.*?)
                    // 所以这里只提取 room 子对象
                    const room = roomInfo.room || roomInfo;
                    const plainRoom = toPlainObject(room, 3);
                    data.room_store = JSON.stringify(plainRoom);

                    console.log('  标题:', data.title || '(未找到)');
                    console.log('  room_id:', room.id_str || roomInfo.roomId || '(未找到)');
                    console.log('  room_store 长度:', data.room_store.length, '字符');
                }

                // 2. 尝试从多个位置提取 user_unique_id（访问者的唯一ID）
                //    这个ID用于生成 WebSocket 连接签名，与主播ID无关

                // 2.1 尝试从页面脚本中提取 user_unique_id（最接近原 HTTP 方式）
                // 原来的正则：user_unique_id\\":\\"(.*?)\\"
                const pageHtml = document.documentElement.outerHTML;
                const uniqueIdMatch = pageHtml.match(/user_unique_id[\\"]?:[\\"]?["']?(\d+)["']?/);
                if (uniqueIdMatch) {
                    data.user_unique_id = uniqueIdMatch[1];
                    console.log('  从页面 HTML 提取 user_unique_id:', data.user_unique_id);
                }

                // 2.2 尝试从 Cookie 中提取 msToken 或其他标识（备选）
                if (!data.user_unique_id) {
                    const cookies = document.cookie.split(';');
                    for (let cookie of cookies) {
                        const [name, value] = cookie.trim().split('=');
                        if (name === 'msToken' && value && value.length > 10) {
                            // 使用 msToken 的一部分作为 unique_id
                            data.user_unique_id = value.substring(0, 16);
                            console.log('  从 Cookie msToken 提取 user_unique_id');
                            break;
                        }
                    }
                }

                // 2.3 尝试从 userStore（备选）
                if (!data.user_unique_id && store.userStore && store.userStore.userInfo) {
                    const userInfo = store.userStore.userInfo;
                    data.user_unique_id = userInfo.id_str || userInfo.web_rid || userInfo.display_id || '';
                    if (data.user_unique_id) {
                        console.log('  从 userStore 提取 user_unique_id:', data.user_unique_id);
                    }
                }

                // 2.4 如果还是没有，生成一个随机的 unique_id（游客模式）
                if (!data.user_unique_id) {
                    // 生成 16 位数字ID（模拟游客ID）
                    data.user_unique_id = Math.floor(Math.random() * 1e16).toString();
                    console.log('  生成随机 user_unique_id (游客模式):', data.user_unique_id);
                }

                // 2.5 提取 ttwid Cookie（用于 WebSocket 认证）
                const cookiesList = document.cookie.split(';');
                for (let cookie of cookiesList) {
                    const [name, value] = cookie.trim().split('=');
                    if (name === 'ttwid') {
                        data.ttwid = value;
                        console.log('  从 Cookie 提取 ttwid:', data.ttwid.substring(0, 20) + '...');
                        break;
                    }
                }

                console.log('✓ 数据提取完成');
                console.log('  - 标题:', data.title ? '已提取' : '未找到');
                console.log('  - user_unique_id:', data.user_unique_id ? '已提取' : '未找到');
                console.log('  - ttwid:', data.ttwid ? '已提取' : '未找到');
                console.log('  - room_store:', data.room_store.length > 0 ? `${data.room_store.length} 字符` : '未找到');

            } catch (e) {
                console.log('❌ 从 __STORE__ 提取数据出错:', e);
            }
        }

        // 如果没有找到有效数据，尝试其他方法
        if (!data.title || !data.room_store) {
            console.log('⚠️  从 __STORE__ 提取失败，尝试备用方法...');

            // 备用方法1: 从 meta 标签提取标题
            if (!data.title) {
                const titleMeta = document.querySelector('meta[property="og:title"]');
                if (titleMeta) {
                    data.title = titleMeta.content;
                    console.log('  从 meta 标签提取标题:', data.title);
                }
            }

            // 备用方法2: 从 document.title 提取
            if (!data.title) {
                data.title = document.title;
                console.log('  从 document.title 提取:', data.title);
            }
        }

        // 输出提取结果
        console.log('🔍 提取结果:');
        console.log('  - 标题:', data.title || '(空)');
        console.log('  - user_unique_id:', data.user_unique_id || '(空)');
        console.log('  - ttwid:', data.ttwid || '(空)');
        console.log('  - room_store 长度:', data.room_store.length);

        // 构建要传递给后端的数据对象
        const resultData = {
            title: data.title,
            user_unique_id: data.user_unique_id,
            ttwid: data.ttwid || '',  // 添加 ttwid 字段
            stream_url: '',  // 保留字段以兼容后端
            room_store: JSON.stringify({
                title: data.title,
                user_unique_id: data.user_unique_id,
                stream_url: ''
            })
        };

        // 如果成功提取了 room_store，使用它
        if (data.room_store && data.room_store.length > 50) {
            resultData.room_store = data.room_store;
        }

        // 通过 URL hash 传递数据给后端
        if (data.title || data.room_store) {
            console.log('✅ 成功提取直播间数据！');
            console.log('📝 标题:', data.title || '未找到');
            console.log('🎬 user_unique_id:', data.user_unique_id || '未找到');
            console.log('🍪 ttwid:', data.ttwid ? (data.ttwid.substring(0, 20) + '...') : '未找到');
            console.log('📊 room_store 长度:', resultData.room_store.length, '字符');

            // 将数据编码为 URL 安全格式并设置到 hash
            const jsonStr = JSON.stringify(resultData);
            const encodedData = encodeURIComponent(jsonStr);

            console.log('✅ 数据已准备好，正在传递给后端...');
            console.log('📝 URL hash 已设置: #__LIVE_DATA__=[数据]');

            // 设置 hash 触发后端检测
            window.location.hash = '__LIVE_DATA__=' + encodedData;

        } else {
            console.log('❌ 未能提取到有效的直播间数据');
        }
    }

    console.log('🚀 开始监听页面数据...');
})();
