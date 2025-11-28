#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
淘宝爬虫依赖自动安装脚本
"""

import sys
import subprocess

def install_package(package):
    """安装 Python 包"""
    print(f"📦 正在安装 {package}...")
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install", package])
        print(f"✅ {package} 安装成功")
        return True
    except subprocess.CalledProcessError:
        print(f"❌ {package} 安装失败")
        return False

def check_and_install(package_name, import_name=None):
    """检查并安装包"""
    if import_name is None:
        import_name = package_name

    try:
        __import__(import_name)
        print(f"✅ {package_name} 已安装")
        return True
    except ImportError:
        print(f"⚠️  {package_name} 未安装")
        return install_package(package_name)

def install_playwright_browsers():
    """安装 Playwright 浏览器"""
    print("\n🌐 安装 Playwright 浏览器...")
    try:
        subprocess.check_call([sys.executable, "-m", "playwright", "install", "chromium"])
        print("✅ Chromium 浏览器安装成功")
        return True
    except subprocess.CalledProcessError:
        print("❌ Chromium 浏览器安装失败")
        return False

def main():
    print("=" * 60)
    print("淘宝爬虫依赖自动安装")
    print("=" * 60)

    # 检查并安装依赖
    packages = [
        ("playwright", "playwright"),
        ("loguru", "loguru"),
        ("aiohttp", "aiohttp"),
    ]

    all_ok = True
    for package_name, import_name in packages:
        if not check_and_install(package_name, import_name):
            all_ok = False

    # 安装 Playwright 浏览器
    if all_ok:
        if not install_playwright_browsers():
            all_ok = False

    print("\n" + "=" * 60)
    if all_ok:
        print("🎉 所有依赖安装完成！")
        print("\n现在可以启动淘宝爬虫了：")
        print("  python taobao_crawler.py --room_id 直播间ID")
    else:
        print("⚠️  部分依赖安装失败")
        print("\n请手动安装：")
        print("  pip install playwright loguru aiohttp")
        print("  playwright install chromium")
    print("=" * 60)

    return 0 if all_ok else 1

if __name__ == "__main__":
    sys.exit(main())
