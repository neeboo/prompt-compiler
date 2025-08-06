#!/usr/bin/env python3
"""
PC Node Client - 封装PC Node的API调用
"""

import requests
import json
import time
from typing import List, Dict, Any, Optional
from dataclasses import dataclass


@dataclass
class ConversationResult:
    """对话结果数据类"""
    content: str
    tokens: int
    response_time: float
    compression_ratio: Optional[float] = None
    context_size: Optional[int] = None


class PCNodeClient:
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.session = requests.Session()

    def health_check(self) -> bool:
        """健康检查"""
        try:
            response = self.session.get(f"{self.base_url}/health")
            return response.status_code == 200
        except Exception:
            return False

    def chat_completion(
        self,
        messages: List[Dict[str, str]],
        agent_id: Optional[str] = None,
        context_sharing: bool = False,
        model: str = "gpt-3.5-turbo",
        temperature: float = 0.7,
        max_tokens: int = 150
    ) -> ConversationResult:
        """发送聊天完成请求"""

        payload = {
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens
        }

        if context_sharing and agent_id:
            payload["context_sharing"] = True
            payload["agent_id"] = agent_id

        start_time = time.time()
        response_time = 0.0  # 初始化默认值

        try:
            response = self.session.post(
                f"{self.base_url}/v1/chat/completions",
                json=payload,
                headers={"Content-Type": "application/json"}
            )

            response_time = time.time() - start_time

            if response.status_code == 200:
                data = response.json()

                return ConversationResult(
                    content=data['choices'][0]['message']['content'],
                    tokens=data['usage']['total_tokens'],
                    response_time=response_time,
                    compression_ratio=data.get('compression_ratio'),
                    context_size=data.get('context_size')
                )
            else:
                raise Exception(f"API error: {response.status_code} - {response.text}")

        except Exception as e:
            response_time = time.time() - start_time  # 确保response_time有值
            print(f"❌ Request failed: {e}")
            return ConversationResult(
                content="",
                tokens=0,
                response_time=response_time,
                compression_ratio=None,
                context_size=None
            )

    def multi_turn_conversation(
        self,
        initial_message: str,
        follow_up_messages: List[str],
        agent_id: Optional[str] = None,
        context_sharing: bool = False
    ) -> List[ConversationResult]:
        """执行多轮对话"""

        results = []
        messages = [{"role": "user", "content": initial_message}]

        # 第一轮对话
        result = self.chat_completion(
            messages=messages,
            agent_id=agent_id,
            context_sharing=context_sharing
        )
        results.append(result)

        if result.content:
            messages.append({"role": "assistant", "content": result.content})

        # 后续轮次
        for follow_up in follow_up_messages:
            messages.append({"role": "user", "content": follow_up})

            result = self.chat_completion(
                messages=messages,
                agent_id=agent_id,
                context_sharing=context_sharing
            )
            results.append(result)

            if result.content:
                messages.append({"role": "assistant", "content": result.content})

        return results
