#!/bin/bash

# 一键 Git 提交推送脚本
# 使用方法: ./git-push.sh [可选的提交消息]

# 默认提交消息
DEFAULT_MESSAGE="自动提交：$(date '+%Y-%m-%d %H:%M:%S')"

# 使用提供的消息或默认消息
COMMIT_MESSAGE="${1:-$DEFAULT_MESSAGE}"

echo "🚀 开始一键 Git 操作..."

# 1. 添加所有更改
echo "📦 git add ."
git add .

# 2. 检查是否有更改
if git diff --staged --quiet; then
    echo "ℹ️  没有更改需要提交"
    echo "🔄 检查是否需要推送..."
    if git status | grep -q "您的分支领先"; then
        echo "🚀 推送现有提交..."
        git push origin main
        echo "✅ 推送完成！"
    else
        echo "✨ 仓库已是最新状态"
    fi
else
    # 3. 提交更改
    echo "💾 git commit -m \"$COMMIT_MESSAGE\""
    git commit -m "$COMMIT_MESSAGE"

    # 4. 推送到远程
    echo "🚀 git push origin main"
    git push origin main

    echo "✅ 完成！所有更改已提交并推送到 GitHub"
fi
