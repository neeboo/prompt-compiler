#!/bin/bash

# GitBook 部署脚本
# 用于快速启动和部署 Prompt Compiler 文档

set -e

echo "🚀 Prompt Compiler GitBook 部署脚本"
echo "=================================="

# 检查GitBook CLI是否已安装
if ! command -v gitbook &> /dev/null; then
    echo "❌ GitBook CLI 未安装"
    echo "请运行以下命令安装："
    echo "npm install -g gitbook-cli"
    exit 1
fi

# 进入docs目录
cd "$(dirname "$0")/docs"

echo "📁 当前目录: $(pwd)"

# 安装GitBook插件
echo "🔧 安装GitBook插件..."
gitbook install

# 构建GitBook
echo "🏗️ 构建GitBook..."
gitbook build

# 启动本地服务器
echo "🌐 启动本地GitBook服务器..."
echo "访问地址: http://localhost:4000"
echo "按 Ctrl+C 停止服务器"

gitbook serve --port 4000
