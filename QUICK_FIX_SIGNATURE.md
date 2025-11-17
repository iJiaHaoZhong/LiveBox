# 快速修复：手动获取签名

## 问题

PyExecJS 在某些环境下无法正确执行抖音的 JavaScript 加密库，导致签名生成失败。

## 🎯 快速解决方案（5分钟）

### 步骤 1: 在浏览器中获取真实签名

1. **打开抖音直播间**
   ```
   https://live.douyin.com/816699487040
   ```

2. **打开开发者工具**
   - Windows/Linux: 按 `F12`
   - macOS: 按 `Cmd + Option + I`

3. **切换到 Network 标签**
   - 点击顶部的 "Network" / "网络" 标签

4. **过滤 WebSocket 连接**
   - 在过滤框中输入 `WS` 或点击 `WS` 按钮
   - 刷新页面（F5）等待直播间加载

5. **找到 WebSocket 连接**
   - 在列表中找到 `im/push/v2/` 开头的连接
   - 点击该连接

6. **复制签名**
   - 在右侧的详情面板中，找到 "Headers" / "标头"
   - 找到 "Request URL" / "请求URL"
   - 复制完整的 URL

   URL 示例：
   ```
   wss://webcast5-ws-web-lf.douyin.com/webcast/im/push/v2/?room_id=7573619563361307442&compress=gzip&...&signature=XXXXXXXXXXXXX&...
   ```

7. **提取 signature 参数**
   - 找到 URL 中的 `signature=` 部分
   - 复制 `signature=` 后面到下一个 `&` 之前的字符串
   - 例如: `DFSzswVLQDw0tCrSSWOJl0QpC35rJptlWv4a`

### 步骤 2: 使用手动签名版本的脚本

运行我为你准备的简化版脚本：

```bash
python douyin_chat_monitor_manual.py
```

程序会提示你输入：
1. 直播间 URL
2. 手动获取的签名

就这么简单！

---

## 📸 图文教程

### 1. 打开开发者工具

![开发者工具](https://via.placeholder.com/800x400?text=F12+Open+DevTools)

### 2. 切换到 Network → WS

![Network WS](https://via.placeholder.com/800x400?text=Network+Tab+%E2%86%92+WS+Filter)

### 3. 找到 WebSocket 连接

![WebSocket Connection](https://via.placeholder.com/800x400?text=Find+im%2Fpush%2Fv2%2F+Connection)

### 4. 复制 signature

![Copy Signature](https://via.placeholder.com/800x400?text=Copy+signature+parameter)

---

## ⚠️ 注意事项

### 签名的有效期

- **时效性**: 签名通常有效期为 **几小时到几天**
- **直播间绑定**: 签名可能与特定直播间绑定
- **过期症状**: 连接被拒绝（HTTP 200 或 403）

### 更新签名

当签名过期时：
1. 重复上述步骤获取新签名
2. 更新代码中的签名字符串
3. 重新运行程序

### 自动化解决方案

如果需要长期运行，建议使用：
1. **Selenium/Playwright** - 自动化浏览器获取签名
2. **定时任务** - 每隔几小时自动刷新签名
3. **多直播间轮换** - 降低被检测的风险

---

## 🔍 常见问题

### Q1: 找不到 WebSocket 连接？

**解决**:
- 确保直播间正在直播中
- 刷新页面（F5）
- 等待几秒让直播间完全加载
- 查看是否有错误提示

### Q2: 复制的 URL 太长？

**解决**:
- 只需要 `signature=` 后面的部分
- 可以用文本编辑器搜索 `signature=`
- 提取该参数的值即可

### Q3: 签名立即失效？

**可能原因**:
- IP 地址不匹配（浏览器和 Python 使用不同 IP）
- 浏览器指纹不匹配
- Cookie 未携带

**解决**:
- 在同一台机器上操作
- 同时复制 Cookie（ttwid）
- 使用相同的 User-Agent

### Q4: 能用多久？

**经验值**:
- 最短: 几分钟（被检测）
- 一般: 2-6 小时
- 最长: 24 小时

建议每次使用前重新获取签名。

---

## 💡 进阶技巧

### 技巧 1: 保存签名到配置文件

创建 `config.json`:
```json
{
  "signature": "你的签名",
  "ttwid": "你的ttwid",
  "room_id": "7573619563361307442",
  "last_update": "2025-11-17 10:30:00"
}
```

程序启动时读取配置。

### 技巧 2: 签名缓存和自动更新

```python
import json
from datetime import datetime, timedelta

def load_signature():
    try:
        with open('config.json', 'r') as f:
            config = json.load(f)

        # 检查是否过期（6小时）
        last_update = datetime.fromisoformat(config['last_update'])
        if datetime.now() - last_update > timedelta(hours=6):
            print("签名可能已过期，请更新")

        return config['signature']
    except:
        return None
```

### 技巧 3: 使用浏览器自动化

```python
from selenium import webdriver

def auto_get_signature(live_url):
    driver = webdriver.Chrome()
    driver.get(live_url)

    # 等待 WebSocket 连接
    time.sleep(5)

    # 获取 performance 日志
    logs = driver.get_log('performance')

    for entry in logs:
        # 解析日志，提取 WebSocket URL
        # 从 URL 中提取 signature
        pass

    driver.quit()
    return signature
```

---

## 📚 相关文档

- [签名算法详解](SIGNATURE_EXPLANATION.md)
- [Python 实现说明](PYTHON_IMPLEMENTATION.md)
- [完整使用指南](SETUP_GUIDE.md)

---

**更新时间**: 2025-11-17
**适用版本**: 所有 Python 实现
**难度**: ⭐ (非常简单)
**时间**: 5 分钟
