// Cookie 自动提取脚本
// 这个脚本会注入到登录窗口中，自动检测登录状态并提取 Cookie

(function() {
    console.log('🔧 Cookie 自动提取脚本已加载');

    let loginDetected = false;
    let checkCount = 0;
    const MAX_CHECKS = 300; // 最多检查 5 分钟 (每秒检查一次)
    const MIN_WAIT_TIME = 5; // 最少等待 5 秒后才开始提取 Cookie（给页面足够加载时间）

    // 检查是否已登录的函数
    function checkLoginStatus() {
        checkCount++;

        // 检查是否超过最大检查次数
        if (checkCount > MAX_CHECKS) {
            console.log('⏱ 超时：未检测到登录');
            clearInterval(loginCheckInterval);
            return;
        }

        // 强制最小等待时间：防止在页面还在加载时就提取 Cookie
        if (checkCount < MIN_WAIT_TIME) {
            if (checkCount === 1) {
                console.log(`⏳ 强制等待 ${MIN_WAIT_TIME} 秒，确保页面完全加载...`);
            }
            return;
        }

        // 获取当前页面的所有 Cookie
        const cookies = document.cookie;

        // 检查页面是否已经不是验证码页面了（验证码完成后会跳转）
        const currentUrl = window.location.href;
        const pageTitle = document.title || '';
        const pageHtml = document.body ? document.body.innerHTML : '';

        // 页面加载保护：确保页面已经加载完成，不是空白页或加载中
        const pageHtmlLength = pageHtml.length;
        const isPageLoaded = pageHtmlLength > 1000; // 至少1000字符，说明页面有实际内容

        // 排除 about:blank 等非目标页面
        const isValidUrl = currentUrl.includes('douyin.com') && !currentUrl.includes('about:blank');

        if (!isValidUrl || !isPageLoaded) {
            if (checkCount % 10 === 0) {
                console.log(`⏳ 页面正在加载... (${checkCount}秒)`);
                console.log(`   - 当前 URL: ${currentUrl}`);
                console.log(`   - 页面长度: ${pageHtmlLength} 字符`);
                console.log(`   - URL 有效: ${isValidUrl}, 页面已加载: ${isPageLoaded}`);
            }
            return; // 页面未加载完成，直接返回，等待下次检查
        }

        const isOnCaptchaPage = pageTitle.includes('验证码') ||
                               pageHtml.includes('验证码中间页') ||
                               pageHtml.includes('middle_page_loading') ||
                               pageHtml.includes('TTGCaptcha');

        // 每 5 秒输出一次详细状态（用于调试）
        if (checkCount === MIN_WAIT_TIME || checkCount % 5 === 0) {
            console.log(`\n========== 状态检查 (${checkCount}秒) ==========`);
            console.log(`URL: ${currentUrl}`);
            console.log(`标题: ${pageTitle}`);
            console.log(`HTML 长度: ${pageHtmlLength}`);
            console.log(`是否在验证码页面: ${isOnCaptchaPage}`);
            console.log(`Cookie 数量: ${cookies.split(';').length}`);
            console.log(`页面已加载: ${isPageLoaded}`);
            console.log(`URL 有效: ${isValidUrl}`);
            console.log(`===================================\n`);
        }

        // 只有在【不在验证码页面】时才检测Cookie
        // 这样可以避免提取到还没有通过验证的旧Cookie
        if (!isOnCaptchaPage && !loginDetected) {
            // 检查当前 URL 是否为有效的抖音域名（必须是实际的抖音页面，不是中间跳转页）
            const isOnDouyinDomain = currentUrl.includes('live.douyin.com') ||
                                     currentUrl.includes('www.douyin.com');

            // 检查是否有登录相关的 Cookie
            const hasSessionId = cookies.includes('sessionid=');
            const hasPassportToken = cookies.includes('passport_auth_token=');
            const hasOdinToken = cookies.includes('odin_tt=');
            const hasSignature = cookies.includes('__ac_signature=');

            const cookieCount = cookies.split(';').length;

            // 检查页面内容是否包含实际的直播间元素（不是中间页或错误页）
            const hasLiveRoomContent = pageHtml.includes('live_room') ||
                                      pageHtml.includes('room_data') ||
                                      pageHtml.includes('webcast');

            // 严格条件：必须满足以下所有条件才提取 Cookie
            // 1. 在抖音域名上
            // 2. 不在验证码页面
            // 3. 有一定数量的 Cookie (>=20)
            // 4. 页面包含直播间相关内容 OR 有关键验证 Cookie
            const shouldExtractCookies = isOnDouyinDomain &&
                                        cookieCount >= 20 &&
                                        (hasLiveRoomContent || hasSessionId || hasPassportToken || hasOdinToken || hasSignature);

            if (shouldExtractCookies) {
                loginDetected = true;
                console.log('✅ 检测到验证码验证完成或登录成功！');
                console.log('🍪 Cookie 数量:', cookieCount);
                console.log('📍 当前页面:', currentUrl);
                console.log('📝 页面标题:', pageTitle);
                console.log('🔍 已确认不在验证码页面');
                console.log('🔍 页面包含直播间内容:', hasLiveRoomContent);
                console.log('🔍 有关键验证 Cookie:', hasSessionId || hasPassportToken || hasOdinToken || hasSignature);
                console.log('✅ Cookie 条件满足，开始保存');

                // 自动保存 Cookie
                saveCookies(cookies);

                // 停止检查
                clearInterval(loginCheckInterval);
            } else if (checkCount % 10 === 0) {
                console.log(`⏳ 已离开验证码页面，但条件未满足，继续等待... (${checkCount}秒)`);
                console.log(`   - 在抖音域名: ${isOnDouyinDomain}`);
                console.log(`   - Cookie 数量: ${cookieCount}`);
                console.log(`   - 有直播间内容: ${hasLiveRoomContent}`);
                console.log(`   - 有验证 Cookie: ${hasSessionId || hasPassportToken || hasOdinToken || hasSignature}`);
            }
        } else if (checkCount % 10 === 0) {
            // 每 10 秒输出一次检查状态
            console.log(`⏳ 等待验证码验证... (${checkCount}秒)`);
            console.log(`   当前页面: ${currentUrl}`);
            console.log(`   页面标题: ${pageTitle}`);
            console.log(`   是否在验证码页面: ${isOnCaptchaPage}`);
        }
    }

    // 保存 Cookie 到全局变量供 Rust 端读取
    async function saveCookies(cookieString) {
        try {
            console.log('💾 正在保存 Cookie...');

            // 检查当前 URL 是否为有效的抖音域名
            const currentUrl = window.location.href;
            const isValidDomain = currentUrl.includes('douyin.com') ||
                                 currentUrl.includes('www.douyin.com') ||
                                 currentUrl.includes('live.douyin.com');

            if (!isValidDomain) {
                console.log('⏳ 当前页面不是抖音域名 (about:blank 或其他)，等待导航到正确页面...');
                return;
            }

            // 将 Cookie 写入 URL hash 供 Rust 端读取
            // 使用 URL hash 是可靠的 IPC 机制，因为 Rust 可以通过 window.url() 读取
            const encodedCookies = encodeURIComponent(cookieString);
            window.location.hash = '__COOKIES__=' + encodedCookies;

            console.log('✅ Cookie 已准备好，正在传递给后端...');
            console.log('🔍 Cookie 数量:', cookieString.split(';').length);
            console.log('📝 URL hash 已设置: #__COOKIES__=[Cookie 数据]');
            console.log('📝 当前 URL:', window.location.href.substring(0, 100) + '...');

            // 显示成功提示
            showSuccessMessage();

            console.log('⏳ 等待 Rust 端读取 Cookie 并关闭窗口...');

        } catch (error) {
            console.error('❌ Cookie 处理失败:', error);
            showErrorMessage(error.toString());
        }
    }

    // 显示成功提示
    function showSuccessMessage() {
        const messageDiv = document.createElement('div');
        messageDiv.style.cssText = `
            position: fixed;
            top: 20px;
            left: 50%;
            transform: translateX(-50%);
            background: #10b981;
            color: white;
            padding: 16px 24px;
            border-radius: 8px;
            font-size: 16px;
            font-weight: bold;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
            z-index: 999999;
            animation: slideDown 0.3s ease-out;
        `;
        messageDiv.innerHTML = `
            <div style="display: flex; align-items: center; gap: 10px;">
                <span style="font-size: 24px;">✅</span>
                <div>
                    <div>登录成功！Cookie 已自动保存</div>
                    <div style="font-size: 12px; opacity: 0.9; margin-top: 4px;">窗口即将自动关闭</div>
                </div>
            </div>
        `;

        // 安全地添加到 DOM
        function addToDOMSafe() {
            if (document.head) {
                const style = document.createElement('style');
                style.textContent = `
                    @keyframes slideDown {
                        from {
                            opacity: 0;
                            transform: translateX(-50%) translateY(-20px);
                        }
                        to {
                            opacity: 1;
                            transform: translateX(-50%) translateY(0);
                        }
                    }
                `;
                document.head.appendChild(style);
            }
            if (document.body) {
                document.body.appendChild(messageDiv);
            }
        }

        if (document.body && document.head) {
            addToDOMSafe();
        } else {
            window.addEventListener('DOMContentLoaded', addToDOMSafe);
        }
    }

    // 显示错误提示
    function showErrorMessage(error) {
        const messageDiv = document.createElement('div');
        messageDiv.style.cssText = `
            position: fixed;
            top: 20px;
            left: 50%;
            transform: translateX(-50%);
            background: #ef4444;
            color: white;
            padding: 16px 24px;
            border-radius: 8px;
            font-size: 16px;
            font-weight: bold;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
            z-index: 999999;
        `;
        messageDiv.innerHTML = `
            <div style="display: flex; align-items: center; gap: 10px;">
                <span style="font-size: 24px;">❌</span>
                <div>
                    <div>Cookie 保存失败</div>
                    <div style="font-size: 12px; opacity: 0.9; margin-top: 4px;">${error}</div>
                </div>
            </div>
        `;

        // 安全地添加到 DOM
        if (document.body) {
            document.body.appendChild(messageDiv);
        } else {
            window.addEventListener('DOMContentLoaded', () => {
                if (document.body) {
                    document.body.appendChild(messageDiv);
                }
            });
        }
    }

    // 显示初始提示
    function showInitialMessage() {
        const messageDiv = document.createElement('div');
        messageDiv.id = 'login-hint-message';
        messageDiv.style.cssText = `
            position: fixed;
            top: 20px;
            left: 50%;
            transform: translateX(-50%);
            background: #3b82f6;
            color: white;
            padding: 16px 24px;
            border-radius: 8px;
            font-size: 16px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
            z-index: 999999;
            animation: slideDown 0.3s ease-out;
        `;
        messageDiv.innerHTML = `
            <div style="display: flex; align-items: center; gap: 10px;">
                <span style="font-size: 24px;">🔐</span>
                <div>
                    <div style="font-weight: bold;">请登录抖音账号</div>
                    <div style="font-size: 12px; opacity: 0.9; margin-top: 4px;">登录成功后 Cookie 会自动保存</div>
                </div>
            </div>
        `;

        // 等待 DOM 加载完成后再添加元素
        function addToDOM() {
            // 添加动画样式
            if (document.head) {
                const style = document.createElement('style');
                style.textContent = `
                    @keyframes slideDown {
                        from {
                            opacity: 0;
                            transform: translateX(-50%) translateY(-20px);
                        }
                        to {
                            opacity: 1;
                            transform: translateX(-50%) translateY(0);
                        }
                    }
                `;
                document.head.appendChild(style);
            }

            // 添加提示消息
            if (document.body) {
                document.body.appendChild(messageDiv);
            }
        }

        // 检查 DOM 是否已准备好
        if (document.body && document.head) {
            addToDOM();
        } else {
            window.addEventListener('DOMContentLoaded', addToDOM);
        }
    }

    // 显示初始提示
    showInitialMessage();

    console.log('🚀 开始监听登录状态...');
    console.log('⏳ 等待页面加载完成（1秒后开始检测）...');

    // 触发验证码检测：主动刷新页面强制重新验证
    // 浏览器直接访问可能绕过验证码，但刷新后会触发验证
    // 使用 sessionStorage 标记，只刷新一次
    const hasRefreshed = sessionStorage.getItem('captcha_refreshed');
    if (!hasRefreshed) {
        setTimeout(() => {
            console.log('🔄 主动刷新页面以触发验证码检测...');
            sessionStorage.setItem('captcha_refreshed', 'true');
            window.location.reload();
        }, 2000);
    } else {
        console.log('✓ 页面已刷新过，开始正常检测');
    }

    // 每秒检查一次登录状态（不立即执行，给页面加载时间）
    const loginCheckInterval = setInterval(checkLoginStatus, 1000);

    // 移除 3 秒延迟检查，避免过早提取 Cookie
    // 等待页面完全加载并导航到正确的直播间页面后，定时器会自动检测并提取
})();
