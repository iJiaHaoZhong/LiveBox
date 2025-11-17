#!/usr/bin/env node
/**
 * 签名生成工具 - Node.js 版本
 * 供 Python 脚本通过子进程调用
 *
 * 使用方法:
 *   node generate_signature.js <room_id> <unique_id>
 */

// 引入签名生成所需的函数（从 vFun.js 复制）
const crypto = require('crypto');

// MD5 相关函数
function wordsToBytes(words) {
    const bytes = [];
    for (let i = 0; i < words.length * 4; i++) {
        bytes.push((words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xFF);
    }
    return bytes;
}

function bytesToHex(bytes) {
    const hex = [];
    for (let i = 0; i < bytes.length; i++) {
        hex.push((bytes[i] >>> 4).toString(16));
        hex.push((bytes[i] & 0xF).toString(16));
    }
    return hex.join('');
}

function stringToBytes(str) {
    const bytes = [];
    for (let i = 0; i < str.length; i++) {
        bytes.push(str.charCodeAt(i));
    }
    return bytes;
}

// 简化的 MD5 实现（使用 Node.js 内置）
function md5Hash(str) {
    return crypto.createHash('md5').update(str).digest('hex');
}

// 简化的签名生成（不依赖 byted_acrawler）
function generateSimpleSignature(roomId, uniqueId) {
    const signStr = `live_id=1,aid=6383,version_code=180800,webcast_sdk_version=1.0.14-beta.0,room_id=${roomId},sub_room_id=,sub_channel_id=,did_rule=3,user_unique_id=${uniqueId},device_platform=web,device_type=,ac=,identity=audience`;

    const hash = md5Hash(signStr);

    // 注意: 这是简化版本，真实签名需要 byted_acrawler
    // 可以尝试留空或使用固定值
    return hash;
}

// 完整的签名生成（需要 byted_acrawler）
function generateFullSignature(roomId, uniqueId) {
    // 如果你有完整的 byted_acrawler 实现，在这里调用
    // 参考 src/assets/static/vFun.js 和 model.js

    // 示例: 返回空字符串（某些情况下可以工作）
    return '';
}

// 主函数
function main() {
    const args = process.argv.slice(2);

    if (args.length < 2) {
        console.error('Usage: node generate_signature.js <room_id> <unique_id>');
        process.exit(1);
    }

    const roomId = args[0];
    const uniqueId = args[1];

    // 生成签名
    const signature = generateSimpleSignature(roomId, uniqueId);

    // 输出签名（供 Python 读取）
    console.log(signature);
}

if (require.main === module) {
    main();
}

module.exports = { generateSimpleSignature, generateFullSignature };
