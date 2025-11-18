// Cookie 自动提取脚本
// 这个脚本会注入到登录窗口中，自动检测登录状态并提取 Cookie

(function() {
    console.log('🔧 Cookie 自动提取脚本已加载');

    let loginDetected = false;
    let checkCount = 0;
    const MAX_CHECKS = 300; // 最多检查 5 分钟 (每秒检查一次)

    // 检查是否已登录的函数
    function checkLoginStatus() {
        checkCount++;

        // 检查是否超过最大检查次数
        if (checkCount > MAX_CHECKS) {
            console.log('⏱ 超时：未检测到登录');
            clearInterval(loginCheckInterval);
            return;
        }

        // 获取当前页面的所有 Cookie
        const cookies = document.cookie;

        // 检查是否有登录相关的 Cookie
        // 抖音登录后通常会有 sessionid, passport_auth_token 等 Cookie
        const hasSessionId = cookies.includes('sessionid=');
        const hasPassportToken = cookies.includes('passport_auth_token=');
        const hasOdinToken = cookies.includes('odin_tt=');
        const hasSignature = cookies.includes('__ac_signature=');

        // 如果检测到任何一个关键 Cookie，说明可能已登录
        if ((hasSessionId || hasPassportToken || hasOdinToken || hasSignature) && !loginDetected) {
            loginDetected = true;
            console.log('✅ 检测到登录！');
            console.log('🍪 Cookie 数量:', cookies.split(';').length);

            // 自动保存 Cookie
            saveCookies(cookies);

            // 停止检查
            clearInterval(loginCheckInterval);
        } else if (checkCount % 10 === 0) {
            // 每 10 秒输出一次检查状态
            console.log(`⏳ 等待登录... (${checkCount}秒)`);
        }
    }

    // 保存 Cookie 到后端
    async function saveCookies(cookieString) {
        try {
            console.log('💾 正在保存 Cookie...');

            // 详细的调试信息
            console.log('🔍 调试信息:');
            console.log('  - window.__TAURI__ 存在?', typeof window.__TAURI__ !== 'undefined');
            console.log('  - window.__TAURI__.invoke 存在?', typeof window.__TAURI__?.invoke !== 'undefined');
            console.log('  - 当前 URL:', window.location.href);
            console.log('  - 窗口名称:', window.name);

            // 检查 Tauri API 是否可用
            if (typeof window.__TAURI__ === 'undefined' || typeof window.__TAURI__.invoke === 'undefined') {
                console.error('❌ Tauri API 不可用！');
                console.error('请确保：');
                console.error('1. 应用已重新编译（npm run tauri dev 或 npm run tauri build）');
                console.error('2. tauri.conf.json 中已配置 dangerousRemoteDomainIpcAccess');
                console.error('3. 域名和窗口标签匹配正确');
                showErrorMessage('Tauri API 不可用，请重新编译应用后再试');
                return;
            }

            // 调用 Tauri 命令保存 Cookie
            const result = await window.__TAURI__.invoke('save_cookies', {
                cookieString: cookieString
            });

            console.log('✅ Cookie 保存成功:', result);

            // 显示成功提示
            showSuccessMessage();

            // 3 秒后自动关闭窗口
            setTimeout(() => {
                console.log('🔒 即将关闭窗口...');
                window.close();
            }, 3000);

        } catch (error) {
            console.error('❌ Cookie 保存失败:', error);
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
                    <div style="font-size: 12px; opacity: 0.9; margin-top: 4px;">窗口将在 3 秒后自动关闭</div>
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

    // 每秒检查一次登录状态
    const loginCheckInterval = setInterval(checkLoginStatus, 1000);

    // 首次立即检查（可能用户已经登录）
    checkLoginStatus();

    console.log('🚀 开始监听登录状态...');
})();
