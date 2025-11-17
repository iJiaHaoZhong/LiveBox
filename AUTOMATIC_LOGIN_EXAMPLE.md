# 自动登录示例代码

## 完全自动化的使用流程

用户**无需手动调用任何命令**，程序会自动处理登录流程！

## 前端实现示例

### 方式一：带自动重试的智能访问

```javascript
import { invoke } from '@tauri-apps/api';

/**
 * 智能访问直播间，自动处理登录
 * @param {string} url - 直播间 URL
 * @param {boolean} autoRetry - 是否在登录后自动重试
 * @returns {Promise<Object>} 直播间信息
 */
async function getLiveInfoSmart(url, autoRetry = true) {
  try {
    // 尝试获取直播间信息
    const liveInfo = await invoke('get_live_html', { url });
    return liveInfo;
  } catch (error) {
    // 检查是否为 Access Denied 错误
    if (error === 'ACCESS_DENIED_NEED_LOGIN') {
      console.log('🔐 需要登录，自动打开登录窗口...');

      // 自动打开登录窗口
      await invoke('open_login_page');

      // 提示用户
      showNotification('请登录抖音账号', '登录成功后将自动继续');

      if (autoRetry) {
        // 等待用户登录（可以监听窗口关闭事件，或者简单等待）
        console.log('⏳ 等待登录完成...');

        // 等待 5 秒后重试（登录窗口会在登录成功 3 秒后关闭）
        await new Promise(resolve => setTimeout(resolve, 5000));

        // 重试获取直播间信息
        console.log('🔄 重试获取直播间信息...');
        return await invoke('get_live_html', { url });
      } else {
        throw new Error('需要登录后手动重试');
      }
    } else {
      // 其他错误直接抛出
      throw error;
    }
  }
}

// 使用示例
async function main() {
  const url = 'https://live.douyin.com/913642684249';

  try {
    const liveInfo = await getLiveInfoSmart(url);
    console.log('✅ 成功获取直播间信息:', liveInfo);
  } catch (error) {
    console.error('❌ 获取失败:', error);
  }
}
```

### 方式二：使用 Promise 链式调用

```javascript
import { invoke } from '@tauri-apps/api';

function getLiveInfo(url) {
  return invoke('get_live_html', { url })
    .catch(error => {
      if (error === 'ACCESS_DENIED_NEED_LOGIN') {
        // 自动打开登录窗口
        return invoke('open_login_page')
          .then(() => {
            // 提示用户登录
            console.log('🔐 请在打开的窗口中登录');
            showNotification('请登录抖音', '登录后将自动保存 Cookie');

            // 返回 Promise，等待用户重新触发
            return new Promise((resolve, reject) => {
              // 可以添加事件监听，在登录成功后自动重试
              reject(new Error('NEED_LOGIN_AND_RETRY'));
            });
          });
      }
      throw error;
    });
}

// 使用示例
getLiveInfo('https://live.douyin.com/913642684249')
  .then(info => {
    console.log('成功:', info);
  })
  .catch(error => {
    if (error.message === 'NEED_LOGIN_AND_RETRY') {
      console.log('请在登录后重新访问');
    } else {
      console.error('错误:', error);
    }
  });
```

### 方式三：React/Vue 组件中的使用

```javascript
import { invoke } from '@tauri-apps/api';
import { useState, useEffect } from 'react';

function LiveRoomComponent({ roomUrl }) {
  const [liveInfo, setLiveInfo] = useState(null);
  const [loading, setLoading] = useState(false);
  const [needLogin, setNeedLogin] = useState(false);

  const fetchLiveInfo = async () => {
    setLoading(true);
    setNeedLogin(false);

    try {
      const info = await invoke('get_live_html', { url: roomUrl });
      setLiveInfo(info);
    } catch (error) {
      if (error === 'ACCESS_DENIED_NEED_LOGIN') {
        // 检测到需要登录
        setNeedLogin(true);

        // 自动打开登录窗口
        try {
          await invoke('open_login_page');
        } catch (e) {
          console.error('打开登录窗口失败:', e);
        }
      } else {
        console.error('获取直播间信息失败:', error);
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchLiveInfo();
  }, [roomUrl]);

  if (loading) {
    return <div>加载中...</div>;
  }

  if (needLogin) {
    return (
      <div>
        <p>需要登录抖音账号</p>
        <p>登录窗口已打开，请在窗口中完成登录</p>
        <button onClick={fetchLiveInfo}>登录完成，重新加载</button>
      </div>
    );
  }

  if (!liveInfo) {
    return <div>暂无数据</div>;
  }

  return (
    <div>
      <h2>直播间信息</h2>
      <pre>{JSON.stringify(liveInfo, null, 2)}</pre>
    </div>
  );
}
```

### 方式四：全局错误处理器

```javascript
import { invoke } from '@tauri-apps/api';

// 全局包装所有 Tauri 命令调用
function createAutoLoginInvoke() {
  const loginPromises = new Map();

  return async function autoLoginInvoke(command, args) {
    try {
      return await invoke(command, args);
    } catch (error) {
      if (error === 'ACCESS_DENIED_NEED_LOGIN') {
        console.log('🔐 需要登录，自动打开登录窗口');

        // 检查是否已经有一个登录流程在进行
        if (!loginPromises.has('current')) {
          const loginPromise = invoke('open_login_page')
            .then(() => {
              console.log('✅ 登录窗口已打开');
              // 等待用户登录
              return new Promise(resolve => setTimeout(resolve, 5000));
            })
            .finally(() => {
              loginPromises.delete('current');
            });

          loginPromises.set('current', loginPromise);
        }

        // 等待登录完成
        await loginPromises.get('current');

        // 重试原命令
        return await invoke(command, args);
      }

      throw error;
    }
  };
}

// 使用示例
const invokeWithAutoLogin = createAutoLoginInvoke();

// 正常使用，就像普通的 invoke
async function getLiveInfo(url) {
  return await invokeWithAutoLogin('get_live_html', { url });
}

// 多个并发请求会共享同一个登录流程
Promise.all([
  getLiveInfo('https://live.douyin.com/room1'),
  getLiveInfo('https://live.douyin.com/room2'),
  getLiveInfo('https://live.douyin.com/room3'),
]).then(results => {
  console.log('全部成功:', results);
});
```

## 用户体验流程

### 第一次使用（未登录）

1. 用户访问直播间
2. 后端检测到 Access Denied
3. 前端收到 `ACCESS_DENIED_NEED_LOGIN` 错误
4. **自动打开登录窗口**（无需用户操作）
5. 用户在窗口中登录
6. Cookie 自动保存
7. 窗口自动关闭
8. 前端自动重试请求
9. 成功获取直播间信息

### 第二次使用（已登录）

1. 用户访问直播间
2. 后端自动加载已保存的 Cookie
3. 直接成功获取直播间信息

**用户全程无需手动输入任何命令或代码！**

## 错误处理最佳实践

```javascript
import { invoke } from '@tauri-apps/api';

class LiveRoomService {
  constructor() {
    this.loginInProgress = false;
    this.retryQueue = [];
  }

  async getLiveInfo(url) {
    try {
      return await invoke('get_live_html', { url });
    } catch (error) {
      if (error === 'ACCESS_DENIED_NEED_LOGIN') {
        return await this.handleLoginRequired(url);
      }
      throw error;
    }
  }

  async handleLoginRequired(url) {
    // 如果已经在登录中，将请求加入队列
    if (this.loginInProgress) {
      return new Promise((resolve, reject) => {
        this.retryQueue.push({ url, resolve, reject });
      });
    }

    try {
      this.loginInProgress = true;

      // 打开登录窗口
      await invoke('open_login_page');

      // 等待登录完成（5秒）
      await new Promise(resolve => setTimeout(resolve, 5000));

      // 重试当前请求
      const result = await invoke('get_live_html', { url });

      // 处理队列中的请求
      this.processRetryQueue();

      return result;
    } catch (error) {
      // 清空队列并传播错误
      this.retryQueue.forEach(item => item.reject(error));
      this.retryQueue = [];
      throw error;
    } finally {
      this.loginInProgress = false;
    }
  }

  async processRetryQueue() {
    const queue = [...this.retryQueue];
    this.retryQueue = [];

    for (const { url, resolve, reject } of queue) {
      try {
        const result = await invoke('get_live_html', { url });
        resolve(result);
      } catch (error) {
        reject(error);
      }
    }
  }
}

// 使用示例
const liveService = new LiveRoomService();

// 即使并发多个请求，也只会打开一次登录窗口
const rooms = [
  'https://live.douyin.com/room1',
  'https://live.douyin.com/room2',
  'https://live.douyin.com/room3',
];

Promise.all(rooms.map(url => liveService.getLiveInfo(url)))
  .then(results => {
    console.log('所有直播间信息:', results);
  })
  .catch(error => {
    console.error('获取失败:', error);
  });
```

## 后端错误码

| 错误码 | 说明 | 前端处理 |
|--------|------|----------|
| `ACCESS_DENIED_NEED_LOGIN` | 访问被拒绝，需要登录 | 自动打开登录窗口 |
| 其他错误 | 网络错误、解析错误等 | 正常错误处理 |

## 总结

现在用户使用流程非常简单：

1. **首次使用**：访问直播间 → 自动弹出登录窗口 → 登录 → 自动完成
2. **后续使用**：直接访问，无需任何操作

**完全自动化，零手动操作！** 🎉
