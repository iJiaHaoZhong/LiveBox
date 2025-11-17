# LiveBox 编译和使用指南

## 📋 目录

- [系统要求](#系统要求)
- [环境准备](#环境准备)
- [安装依赖](#安装依赖)
- [开发模式运行](#开发模式运行)
- [编译打包](#编译打包)
- [使用推送功能](#使用推送功能)
- [Python 接收服务器](#python-接收服务器)
- [常见问题](#常见问题)

---

## 系统要求

### 操作系统
- Windows 10/11 (推荐)
- macOS 10.15+
- Ubuntu 20.04+ / Debian 11+

### 软件版本
- **Node.js**: 16.x 或更高版本
- **npm**: 8.x 或更高版本
- **Rust**: 1.70+ (Tauri 需要)
- **Python**: 3.8+ (用于接收服务器，可选)

---

## 环境准备

### 1. 安装 Node.js

#### Windows
从 [Node.js 官网](https://nodejs.org/) 下载安装器，选择 LTS 版本。

#### macOS
```bash
# 使用 Homebrew
brew install node
```

#### Linux
```bash
# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# 或使用 nvm (推荐)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
```

### 2. 安装 Rust (Tauri 依赖)

```bash
# Windows/macOS/Linux 通用
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装完成后，重启终端或运行:
source $HOME/.cargo/env
```

### 3. 安装平台特定依赖

#### Windows
安装 [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

#### macOS
```bash
xcode-select --install
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

---

## 安装依赖

克隆项目并安装依赖：

```bash
# 克隆项目（如果还没有）
git clone https://github.com/iJiaHaoZhong/LiveBox.git
cd LiveBox

# 安装 npm 依赖
npm install
```

---

## 开发模式运行

开发模式下可以实时预览修改效果：

```bash
# ✅ 正确：启动 Tauri 桌面应用（推荐）
npm run tauri dev

# ⚠️ 注意：不要使用 npm run dev（只启动浏览器，缺少 Tauri 功能）
```

这会同时启动：
- Vite 开发服务器（前端热重载）
- Tauri 桌面应用窗口（完整的桌面功能）

**注意**：
- 首次运行会编译 Rust 代码，可能需要 5-10 分钟
- 后续运行会快很多（增量编译）
- 必须使用 `npm run tauri dev` 而不是 `npm run dev`，否则会缺少 Tauri IPC 功能

---

## 编译打包

### 构建生产版本

```bash
# 构建完整的应用程序
npm run tauri build

# 或者使用调试版本（更快但文件更大）
npm run tauri debug
```

### 查找编译产物

编译完成后，安装包位置：

#### Windows
```
src-tauri/target/release/bundle/msi/LiveBox_版本号_x64_zh-CN.msi
src-tauri/target/release/bundle/nsis/LiveBox_版本号_x64-setup.exe
```

#### macOS
```
src-tauri/target/release/bundle/dmg/LiveBox_版本号_x64.dmg
src-tauri/target/release/bundle/macos/LiveBox.app
```

#### Linux
```
src-tauri/target/release/bundle/deb/livebox_版本号_amd64.deb
src-tauri/target/release/bundle/appimage/livebox_版本号_amd64.AppImage
```

### 仅构建前端 (无打包)

```bash
# 仅构建 Vue 前端
npm run build:vue

# 输出目录: dist/
```

---

## 使用推送功能

LiveBox 现在支持将接收到的消息推送到你自己的服务器。

### 1. 配置推送地址

在 LiveBox 界面中：

1. 点击 **设置** 按钮
2. 找到 **推送地址** 输入框
3. 输入你的接收服务器地址，例如：
   ```
   http://localhost:5000/webhook
   ```
4. 选择要推送的消息类型（聊天、礼物、点赞、关注、进入）

### 2. 推送消息格式

LiveBox 会向配置的 URL 发送 POST 请求，JSON 格式：

```json
{
  "type": "chat",
  "data": {
    "id": "1234567890",
    "name": "用户昵称",
    "msg": "消息内容"
  },
  "raw": {
    // 完整的原始 Protocol Buffer 数据
  },
  "timestamp": 1700000000000,
  "room_id": "7573619563361307442"
}
```

#### 消息类型

- `chat` - 聊天消息
- `gift` - 礼物消息
- `like` - 点赞消息
- `follow` - 关注消息
- `comein` - 进入直播间消息

---

## Python 接收服务器

项目包含了一个 Python 示例接收服务器 `example_receiver.py`。

### 1. 安装 Python 依赖

```bash
# 创建虚拟环境（推荐）
python -m venv venv

# 激活虚拟环境
# Windows:
venv\Scripts\activate
# macOS/Linux:
source venv/bin/activate

# 安装 Flask
pip install flask
```

### 2. 启动接收服务器

```bash
python example_receiver.py
```

输出：
```
============================================================
LiveBox 消息接收服务器启动
============================================================
Webhook 地址: http://localhost:5000/webhook
健康检查: http://localhost:5000/health
支持的消息类型: chat, gift, like, follow, comein
============================================================

在 LiveBox 中配置推送地址为: http://localhost:5000/webhook
```

### 3. 测试接收

启动 LiveBox 并配置推送地址为 `http://localhost:5000/webhook`，你会在终端看到接收到的消息：

```
============================================================
[2025-11-17 10:30:45] 收到消息
消息类型: chat
直播间ID: 7573619563361307442
时间戳: 1700123445000
消息内容: {
  "id": "123456",
  "name": "测试用户",
  "msg": "你好！"
}
============================================================

[聊天] 测试用户: 你好！
```

### 4. 自定义消息处理

编辑 `example_receiver.py`，在对应的处理函数中添加你的业务逻辑：

```python
@register_handler('chat')
def handle_chat(data):
    """处理聊天消息"""
    print(f"[聊天] {data['name']}: {data['msg']}")

    # 添加你的业务逻辑
    # 例如：保存到数据库
    # save_to_database(data)

    # 例如：关键词触发
    # if '抽奖' in data['msg']:
    #     trigger_lottery()

    return {"status": "ok", "action": "chat_received"}
```

---

## 常见问题

### Q1: npm install 失败

**解决方案**:
```bash
# 清理缓存
npm cache clean --force

# 删除 node_modules 和 package-lock.json
rm -rf node_modules package-lock.json

# 重新安装
npm install

# 如果还是失败，尝试使用 cnpm（中国用户）
npm install -g cnpm --registry=https://registry.npmmirror.com
cnpm install
```

### Q2: Rust 编译失败

**解决方案**:
```bash
# 更新 Rust
rustup update

# 清理 Rust 缓存
cd src-tauri
cargo clean
cd ..

# 重新编译
npm run build
```

### Q3: 开发模式启动后白屏或报错 "window.__TAURI_IPC__ is not a function"

**原因**: 使用了错误的启动命令

**解决方案**:
```bash
# ❌ 不要使用这个（只启动浏览器）
npm run dev

# ✅ 使用这个（启动 Tauri 桌面应用）
npm run tauri dev
```

其他检查：
1. 确认 Node.js 版本 >= 16
2. 确认 Rust 已正确安装
3. 清理并重新安装依赖
4. 检查防火墙是否阻止了本地端口

### Q4: macOS 提示"无法打开，因为无法验证开发者"

**解决方案**:
```bash
# 右键点击应用 -> 选择"打开"
# 或者在终端运行：
sudo xattr -rd com.apple.quarantine /Applications/LiveBox.app
```

### Q5: Linux 提示缺少依赖

**解决方案**:
```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel gtk3-devel libappindicator-gtk3-devel

# Arch Linux
sudo pacman -S webkit2gtk gtk3 libappindicator-gtk3
```

### Q6: 推送功能不工作

**检查清单**:
1. 确认推送地址格式正确（包含 http:// 或 https://）
2. 确认接收服务器已启动
3. 查看浏览器开发者工具的 Console 是否有错误
4. 检查防火墙是否阻止了连接
5. 测试接收服务器健康检查端点：`curl http://localhost:5000/health`

### Q7: Python 接收服务器启动失败

**解决方案**:
```bash
# 检查端口是否被占用
# Windows:
netstat -ano | findstr :5000
# macOS/Linux:
lsof -i :5000

# 如果端口被占用，可以修改 example_receiver.py 中的端口号：
# app.run(host='0.0.0.0', port=5001, debug=True)  # 改为 5001 或其他端口
```

---

## 项目结构

```
LiveBox/
├── src/                    # Vue.js 前端源码
│   ├── App.vue            # 主应用组件（包含推送逻辑）
│   ├── main.ts            # 入口文件
│   └── ...
├── src-tauri/             # Tauri 桌面应用配置
│   ├── src/               # Rust 后端代码
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── public/                # 静态资源
├── dist/                  # 构建输出（前端）
├── example_receiver.py    # Python 接收服务器示例
├── package.json           # npm 依赖配置
├── vite.config.ts         # Vite 构建配置
└── BUILD_GUIDE.md         # 本文档
```

---

## 版本信息

- **LiveBox 版本**: 参见 package.json
- **Tauri 版本**: 1.x
- **Vue 版本**: 3.x
- **Node.js 要求**: >= 16.x

---

## 技术支持

如有问题，请查看：
- [Tauri 文档](https://tauri.app/zh-cn/)
- [Vue.js 文档](https://cn.vuejs.org/)
- [项目 Issues](https://github.com/iJiaHaoZhong/LiveBox/issues)

---

**最后更新**: 2025-11-17
