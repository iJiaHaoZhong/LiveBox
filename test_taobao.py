#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
淘宝爬虫快速测试脚本
用于验证环境配置是否正确
"""

import sys
import subprocess

def check_python():
    """检查 Python 版本"""
    print("🔍 检查 Python 版本...")
    version = sys.version_info
    print(f"✅ Python {version.major}.{version.minor}.{version.micro}")
    if version.major < 3 or (version.major == 3 and version.minor < 7):
        print("❌ 需要 Python 3.7 或更高版本")
        return False
    return True

def check_module(module_name):
    """检查模块是否已安装"""
    try:
        __import__(module_name)
        print(f"✅ {module_name}")
        return True
    except ImportError:
        print(f"❌ {module_name} 未安装")
        return False

def check_dependencies():
    """检查依赖包"""
    print("\n🔍 检查依赖包...")
    modules = {
        'playwright': 'playwright',
        'loguru': 'loguru',
        'aiohttp': 'aiohttp',
    }

    all_ok = True
    for display_name, module_name in modules.items():
        if not check_module(module_name):
            all_ok = False

    return all_ok

def check_playwright_browsers():
    """检查 Playwright 浏览器"""
    print("\n🔍 检查 Playwright 浏览器...")
    try:
        result = subprocess.run(
            ['playwright', 'install', '--dry-run', 'chromium'],
            capture_output=True,
            text=True,
            timeout=10
        )
        if 'is already installed' in result.stdout or result.returncode == 0:
            print("✅ Chromium 浏览器已安装")
            return True
        else:
            print("⚠️  Chromium 浏览器可能未安装")
            print("💡 运行: playwright install chromium")
            return False
    except subprocess.TimeoutExpired:
        print("⏱  检查超时")
        return False
    except FileNotFoundError:
        print("❌ playwright 命令未找到")
        print("💡 运行: pip install playwright")
        return False

def test_import():
    """测试导入 taobao_crawler"""
    print("\n🔍 测试导入 taobao_crawler.py...")
    try:
        import taobao_crawler
        print("✅ taobao_crawler.py 可以导入")
        return True
    except Exception as e:
        print(f"❌ 导入失败: {e}")
        return False

def main():
    """主函数"""
    print("=" * 60)
    print("淘宝爬虫环境检查")
    print("=" * 60)

    checks = [
        ("Python 版本", check_python),
        ("依赖包", check_dependencies),
        ("Playwright 浏览器", check_playwright_browsers),
        ("导入测试", test_import),
    ]

    results = []
    for name, check_func in checks:
        try:
            result = check_func()
            results.append((name, result))
        except Exception as e:
            print(f"❌ {name} 检查失败: {e}")
            results.append((name, False))

    print("\n" + "=" * 60)
    print("检查结果汇总")
    print("=" * 60)

    all_passed = True
    for name, result in results:
        status = "✅ 通过" if result else "❌ 失败"
        print(f"{name}: {status}")
        if not result:
            all_passed = False

    print("=" * 60)

    if all_passed:
        print("🎉 所有检查通过！您可以开始使用淘宝爬虫了")
        print("\n使用方法:")
        print("  python3 taobao_crawler.py --room_id 直播间ID")
    else:
        print("⚠️  部分检查未通过，请先解决上述问题")
        print("\n安装依赖:")
        print("  pip install playwright loguru aiohttp")
        print("  playwright install chromium")

    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())
