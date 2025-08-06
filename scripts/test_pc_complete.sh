#!/bin/bash

# PC Node 启动和测试脚本

echo "🚀 PC Node 启动和测试脚本"
echo "=================================="

# 检查环境变量
if [ -z "$OPENAI_API_KEY" ] && [ -z "$LLM_API_KEY" ]; then
    echo "⚠️  警告: 未设置 OPENAI_API_KEY 或 LLM_API_KEY 环境变量"
    echo "请设置其中一个："
    echo "  export OPENAI_API_KEY='your-api-key-here'"
    echo "  或者"
    echo "  export LLM_API_KEY='your-api-key-here'"
    echo ""
    echo "如果没有API密钥，测试将会失败。"
    echo "是否继续? (y/N)"
    read -r response
    if [[ ! "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        echo "已取消"
        exit 0
    fi
fi

# 设置默认端口
export PORT=${PORT:-3000}

echo "🔧 环境配置:"
echo "  端口: $PORT"
echo "  API密钥: ${OPENAI_API_KEY:+已设置}${LLM_API_KEY:+已设置}${OPENAI_API_KEY:-${LLM_API_KEY:-未设置}}"
echo ""

# 编译并启动PC Node（后台运行）
echo "🔨 编译PC Node..."
cargo build --package prompt-compiler-node
if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi

echo "🚀 启动PC Node服务器..."
cargo run --package prompt-compiler-node &
SERVER_PID=$!

echo "📋 服务器PID: $SERVER_PID"

# 等待服务器启动
echo "⏳ 等待服务器启动..."
sleep 5

# 检查服务器是否启动成功
if curl -s http://localhost:$PORT/health > /dev/null 2>&1; then
    echo "✅ 服务器启动成功！"
else
    echo "⚠️  服务器可能还在启动中，继续等待..."
    sleep 3
fi

# 运行测试
echo ""
echo "🧪 运行测试套件..."
echo "=================================="
python scripts/test_pc_node.py --url "http://localhost:$PORT"

# 清理
echo ""
echo "🧹 清理资源..."
echo "正在停止服务器 (PID: $SERVER_PID)..."
kill $SERVER_PID 2>/dev/null || true

# 等待进程结束
sleep 2

# 强制清理任何剩余的进程
pkill -f "prompt-compiler-node" 2>/dev/null || true

echo "✅ 测试完成！"
