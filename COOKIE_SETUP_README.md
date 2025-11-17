# Cookie 功能使用说明

## 功能概述

为了解决抖音直播间访问时出现的 "Access Denied" 错误，我们添加了 Cookie 管理功能。用户可以通过手动登录抖音账号，然后保存 Cookie 到本地，程序会自动使用这些 Cookie 来访问直播间。

## 新增功能

### 1. Cookie 持久化存储

- Cookie 保存位置：`~/.livebox/douyin_cookies.json` (Linux/macOS) 或 `C:\Users\用户名\.livebox\douyin_cookies.json` (Windows)
- 自动加载：程序会在访问直播间时自动加载已保存的 Cookie
- JSON 格式存储，便于查看和管理

### 2. Tauri 命令

新增了以下 4 个 Tauri 命令：

#### `save_cookies(cookie_string: String)`
保存用户提供的 Cookie 字符串到本地文件。

**参数：**
- `cookie_string`: Cookie 字符串，格式为 `name1=value1; name2=value2`

**返回：**
- 成功：`"成功保存 N 个 cookies 到 路径"`
- 失败：错误信息

**使用示例：**
```javascript
import { invoke } from '@tauri-apps/api';

const cookieString = "__ac_nonce=06123abc; ttwid=1%7CAbCdEf; sessionid=xyz123";
const result = await invoke('save_cookies', { cookieString });
console.log(result);
```

#### `load_cookies()`
从本地文件加载已保存的 Cookie。

**返回：**
- 成功：Cookie 字符串
- 失败：错误信息

**使用示例：**
```javascript
const cookieString = await invoke('load_cookies');
console.log('已加载 Cookie:', cookieString);
```

#### `clear_cookies()`
清除本地保存的 Cookie 文件。

**返回：**
- 成功：`"成功清除 cookies"`
- 失败：错误信息

**使用示例：**
```javascript
await invoke('clear_cookies');
```

#### `open_login_page()`
打开一个抖音登录页面的窗口，方便用户登录并复制 Cookie。

**返回：**
- 成功：`"登录窗口已打开，请在浏览器中登录抖音"`
- 失败：错误信息

**使用示例：**
```javascript
await invoke('open_login_page');
```

## 使用流程

### 完整流程示例

```javascript
import { invoke } from '@tauri-apps/api';

// 1. 打开登录窗口（可选）
await invoke('open_login_page');

// 2. 用户在打开的窗口中登录，然后在浏览器控制台运行：
// copy(document.cookie);
// 复制剪贴板中的 Cookie

// 3. 保存 Cookie
const cookieString = "从浏览器复制的 Cookie 字符串";
await invoke('save_cookies', { cookieString });

// 4. 之后访问直播间时，程序会自动使用保存的 Cookie
const liveInfo = await invoke('get_live_html', { url: 'https://live.douyin.com/913642684249' });
```

## 如何获取 Cookie

### 方法一：使用 `open_login_page` 命令

1. 在应用中调用 `open_login_page` 命令
2. 在打开的窗口中登录抖音账号
3. 登录成功后，按 F12 打开开发者工具
4. 切换到 Console (控制台) 标签
5. 运行以下代码：
   ```javascript
   copy(document.cookie);
   console.log('Cookie 已复制到剪贴板！');
   ```
6. Cookie 已自动复制到剪贴板，可以直接粘贴使用

### 方法二：在浏览器中手动获取

1. 使用 Chrome/Edge/Firefox 浏览器访问 https://www.douyin.com
2. 登录你的抖音账号
3. 按 F12 打开开发者工具
4. 切换到 Console (控制台) 标签
5. 运行 `copy(document.cookie);`
6. Cookie 已复制到剪贴板

## 技术实现

### 文件结构

```
src-tauri/src/
├── command/
│   ├── cookie.rs          # Cookie 管理命令
│   ├── runner.rs          # 修改：加载和使用 Cookie
│   └── mod.rs             # 添加 cookie 模块
├── utils/
│   ├── cookie_store.rs    # Cookie 存储和加载逻辑
│   └── mod.rs
└── main.rs                # 注册新命令
```

### Cookie 数据结构

```rust
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

pub struct CookieStore {
    pub cookies: Vec<CookieData>,
}
```

### 自动加载逻辑

在 `runner.rs` 的 `get_room_info` 方法中：

```rust
// 尝试加载用户保存的 Cookie
let saved_cookies = if let Ok(cookie_path) = CookieStore::get_default_path() {
    if cookie_path.exists() {
        match CookieStore::load_from_file(&cookie_path) {
            Ok(store) => {
                println!("✓ 成功加载 {} 个已保存的用户 Cookie", store.cookies.len());
                Some(store.to_cookie_string())
            }
            Err(e) => {
                println!("⚠ 加载保存的 Cookie 失败: {}", e);
                None
            }
        }
    } else {
        println!("ℹ 未找到保存的 Cookie 文件，使用默认请求");
        None
    }
} else {
    None
};

// 如果有保存的 Cookie，添加到请求头
if let Some(cookie_str) = saved_cookies {
    headers.insert("cookie", cookie_str.parse()?);
    println!("✓ 已将保存的 Cookie 添加到请求头");
}
```

## 安全注意事项

1. **Cookie 包含敏感信息**：Cookie 中包含你的登录会话信息，请妥善保管
2. **不要分享 Cookie**：不要将 Cookie 分享给他人，否则他们可以使用你的账号
3. **定期更新**：Cookie 可能会过期，如果出现访问问题，请重新登录并更新 Cookie
4. **安全存储**：Cookie 文件存储在用户目录下，确保文件权限设置正确

## 故障排除

### 问题：仍然出现 "Access Denied"

**可能原因：**
1. Cookie 未正确保存
2. Cookie 已过期
3. Cookie 格式不正确

**解决方案：**
1. 检查 `~/.livebox/douyin_cookies.json` 文件是否存在
2. 使用 `load_cookies` 命令验证 Cookie 是否正确加载
3. 重新登录并获取新的 Cookie
4. 确认 Cookie 字符串格式正确（应为 `name1=value1; name2=value2` 格式）

### 问题：无法保存 Cookie

**可能原因：**
1. 文件系统权限问题
2. 磁盘空间不足

**解决方案：**
1. 检查 `~/.livebox` 目录是否有写权限
2. 手动创建 `~/.livebox` 目录
3. 检查磁盘空间

## 日志输出

程序会输出以下日志帮助调试：

```
步骤2: 使用 Cookie 访问直播间...
✓ 成功加载 15 个已保存的用户 Cookie
✓ 已将保存的 Cookie 添加到请求头
```

如果没有保存 Cookie：
```
步骤2: 使用 Cookie 访问直播间...
ℹ 未找到保存的 Cookie 文件，使用默认请求
```

## 更多帮助

详细的使用指南请参考 [COOKIE_GUIDE.md](./COOKIE_GUIDE.md)
