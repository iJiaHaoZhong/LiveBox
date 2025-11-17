# 抖音 Cookie 获取和保存指南

## 为什么需要 Cookie？

抖音可能会对未登录的访问进行限制，返回 "Access Denied" 错误。通过使用登录后的 Cookie，可以绕过这些限制，正常访问直播间信息。

## 🎯 最佳使用方式：智能自动检测（零配置）

**用户无需任何额外操作！**

程序会自动检测 Access Denied 错误，并返回特殊错误码。前端捕获后自动打开登录窗口。

**前端示例代码：**
```javascript
async function getLiveInfo(url) {
  try {
    return await invoke('get_live_html', { url });
  } catch (error) {
    if (error === 'ACCESS_DENIED_NEED_LOGIN') {
      // 自动打开登录窗口
      await invoke('open_login_page');
      // 等待用户登录后重试
      return await invoke('get_live_html', { url });
    }
    throw error;
  }
}
```

**详细的自动化示例请查看：[AUTOMATIC_LOGIN_EXAMPLE.md](./AUTOMATIC_LOGIN_EXAMPLE.md)**

## 方法一：手动打开登录窗口（全自动提取）

### 步骤 1: 打开登录窗口

在应用中调用 `open_login_page` 命令，会自动打开一个抖音登录页面的窗口。

```javascript
await invoke('open_login_page');
```

### 步骤 2: 登录抖音账号

在打开的窗口中：
1. 使用手机扫码登录，或使用账号密码登录
2. 确保登录成功

### 步骤 3: 自动完成！

**无需任何手动操作！**

- 登录成功后，程序会**自动检测**到登录状态
- **自动提取** Cookie
- **自动保存** Cookie 到本地
- 显示 "✅ 登录成功！Cookie 已自动保存" 提示
- 3 秒后**自动关闭**登录窗口

**就这么简单！** 用户只需要登录，程序会自动处理剩下的一切。

## 方法二：手动从浏览器获取

### Chrome/Edge 浏览器

1. 打开 Chrome 或 Edge 浏览器
2. 访问 https://www.douyin.com
3. 登录你的抖音账号
4. 打开开发者工具 (F12)
5. 切换到 "Console" (控制台) 标签
6. 运行以下代码复制 Cookie：

```javascript
copy(document.cookie);
console.log('Cookie 已复制到剪贴板！格式示例:');
console.log('name1=value1; name2=value2; name3=value3');
```

7. Cookie 会自动复制到剪贴板

### Firefox 浏览器

1. 打开 Firefox 浏览器
2. 访问 https://www.douyin.com
3. 登录你的抖音账号
4. 打开开发者工具 (F12)
5. 切换到 "Console" (控制台) 标签
6. 运行以下代码：

```javascript
copy(document.cookie);
console.log('Cookie 已复制到剪贴板！');
```

## Cookie 格式示例

Cookie 应该是以下格式的字符串（分号分隔的键值对）：

```
__ac_nonce=06123abc4567def8901234; ttwid=1%7CAbCdEf...; sessionid=xyz123...; odin_tt=abc123...
```

## API 使用示例

### 保存 Cookie

```javascript
import { invoke } from '@tauri-apps/api';

const cookieString = "__ac_nonce=06123abc; ttwid=1%7CAbCdEf; sessionid=xyz123";

try {
  const result = await invoke('save_cookies', { cookieString });
  console.log(result); // "成功保存 3 个 cookies 到 ~/.livebox/douyin_cookies.json"
} catch (error) {
  console.error('保存失败:', error);
}
```

### 加载 Cookie

```javascript
try {
  const cookieString = await invoke('load_cookies');
  console.log('已加载 Cookie:', cookieString);
} catch (error) {
  console.error('加载失败:', error);
}
```

### 清除 Cookie

```javascript
try {
  const result = await invoke('clear_cookies');
  console.log(result);
} catch (error) {
  console.error('清除失败:', error);
}
```

### 打开登录页面

```javascript
try {
  const result = await invoke('open_login_page');
  console.log(result); // "登录窗口已打开，请在浏览器中登录抖音"
} catch (error) {
  console.error('打开失败:', error);
}
```

## Cookie 存储位置

Cookie 会被保存到以下位置：

- **Windows**: `C:\Users\你的用户名\.livebox\douyin_cookies.json`
- **macOS/Linux**: `~/.livebox/douyin_cookies.json`

## 自动使用 Cookie

一旦保存了 Cookie，程序会在调用 `get_live_html` 时自动加载并使用它们，无需额外操作。

你会在控制台看到以下输出：

```
步骤2: 使用 Cookie 访问直播间...
✓ 成功加载 15 个已保存的用户 Cookie
✓ 已将保存的 Cookie 添加到请求头
```

## 注意事项

1. **Cookie 安全性**: Cookie 包含你的登录信息，请妥善保管，不要分享给他人
2. **Cookie 过期**: Cookie 可能会过期，如果访问失败，请重新登录并更新 Cookie
3. **多账号**: 如果需要切换账号，请先清除旧的 Cookie，再保存新账号的 Cookie

## 故障排除

### 问题 1: 仍然出现 "Access Denied"

**解决方案**:
1. 确认已成功登录抖音账号
2. 重新获取并保存 Cookie
3. 检查 Cookie 格式是否正确（应该是 `name1=value1; name2=value2` 格式）
4. 确认 Cookie 文件已成功保存（检查 `~/.livebox/douyin_cookies.json`）

### 问题 2: Cookie 加载失败

**解决方案**:
1. 检查文件路径权限
2. 确认 JSON 文件格式正确
3. 尝试清除并重新保存 Cookie

### 问题 3: 如何验证 Cookie 是否有效？

在浏览器中登录后，访问直播间页面，查看是否能正常显示内容。如果可以，说明 Cookie 有效。

## 更新日志

- 2024-11-17: 添加 Cookie 管理功能，支持用户手动登录获取 Cookie
