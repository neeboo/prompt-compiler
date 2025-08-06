#!/usr/bin/env python3
"""
单智能体多轮对话测试
比较开启和关闭Context Sharing的性能差异
"""

import os
import json
import time
from typing import List, Dict, Any
from datetime import datetime
from utils.pc_client import PCNodeClient, ConversationResult
from utils.performance_metrics import MetricsCalculator
from utils.chart_generator import ChartGenerator


class SingleAgentTester:
    def __init__(self, config_path: str = None):
        if config_path is None:
            # 使用脚本所在目录的相对路径
            script_dir = os.path.dirname(os.path.abspath(__file__))
            config_path = os.path.join(script_dir, "configs", "single_agent_config.json")

        with open(config_path, 'r', encoding='utf-8') as f:
            self.config = json.load(f)

        self.client = PCNodeClient(self.config['test_config']['base_url'])
        self.chart_generator = ChartGenerator()

    def test_without_context_sharing(self) -> List[ConversationResult]:
        """测试不开启Context Sharing的场景"""
        print("🔍 Testing WITHOUT Context Sharing...")

        conversation = self.config['conversation_scenarios']['web_scraping']
        results = []

        # 不使用context sharing，每次都传递完整的消息历史
        messages = []

        for i, message in enumerate(conversation):
            print(f"   Turn {i+1}/{len(conversation)}: {message[:50]}...")

            messages.append({"role": "user", "content": message})

            result = self.client.chat_completion(
                messages=messages,
                context_sharing=False,
                model=self.config['test_config']['model'],
                temperature=self.config['test_config']['temperature'],
                max_tokens=self.config['test_config']['max_tokens']
            )

            results.append(result)

            if result.content:
                messages.append({"role": "assistant", "content": result.content})

            time.sleep(0.1)  # 避免请求过快

        return results

    def test_with_context_sharing(self) -> List[ConversationResult]:
        """测试开启Context Sharing的场景"""
        print("🔍 Testing WITH Context Sharing...")

        conversation = self.config['conversation_scenarios']['web_scraping']
        results = []
        agent_id = "single_agent_tester"

        for i, message in enumerate(conversation):
            print(f"   Turn {i+1}/{len(conversation)}: {message[:50]}...")

            # 使用context sharing，只需要发送当前消息
            result = self.client.chat_completion(
                messages=[{"role": "user", "content": message}],
                agent_id=agent_id,
                context_sharing=True,
                model=self.config['test_config']['model'],
                temperature=self.config['test_config']['temperature'],
                max_tokens=self.config['test_config']['max_tokens']
            )

            results.append(result)
            time.sleep(0.1)

        return results

    def run_comparison_test(self) -> Dict[str, Any]:
        """运行对比测试"""
        print("🚀 Starting Single Agent Comparison Test...")

        # 健康检查
        if not self.client.health_check():
            print("❌ PC Node is not healthy, aborting test")
            return {}

        # 运行两个场景
        start_time = time.time()

        results_without = self.test_without_context_sharing()
        results_with = self.test_with_context_sharing()

        total_time = time.time() - start_time

        # 计算性能指标对比
        comparison = MetricsCalculator.compare_scenarios(
            results_without,
            results_with,
            "Without Context Sharing",
            "With Context Sharing"
        )

        # 生成图表
        chart_path = self.chart_generator.generate_comparison_chart(
            results_without,
            results_with,
            "Without Context Sharing",
            "With Context Sharing",
            "Single Agent Performance Comparison",
            "single_agent_comparison.png"
        )

        # 准备返回结果
        result = {
            "test_type": "single_agent_comparison",
            "total_test_time": total_time,
            "conversation_rounds": len(results_without),
            "comparison": comparison,
            "chart_path": chart_path,
            "raw_results": {
                "without_context_sharing": [
                    {
                        "turn": i+1,
                        "tokens": r.tokens,
                        "response_time": r.response_time,
                        "compression_ratio": r.compression_ratio,
                        "context_size": r.context_size
                    }
                    for i, r in enumerate(results_without)
                ],
                "with_context_sharing": [
                    {
                        "turn": i+1,
                        "tokens": r.tokens,
                        "response_time": r.response_time,
                        "compression_ratio": r.compression_ratio,
                        "context_size": r.context_size
                    }
                    for i, r in enumerate(results_with)
                ]
            }
        }

        self._print_summary(comparison)

        return result

    def _print_summary(self, comparison: Dict[str, Any]):
        """打印测试总结"""
        print("\n" + "="*60)
        print("📊 SINGLE AGENT TEST SUMMARY")
        print("="*60)

        scenarios = comparison["scenarios"]
        improvements = comparison.get("improvements", {})  # 使用安全访问

        print(f"🔸 Without Context Sharing:")
        print(f"   Average Tokens: {scenarios.get('Without Context Sharing', {}).get('avg_tokens', 0):.1f}")
        print(f"   Total Tokens: {scenarios.get('Without Context Sharing', {}).get('total_tokens', 0)}")
        print(f"   Average Response Time: {scenarios.get('Without Context Sharing', {}).get('avg_response_time', 0):.3f}s")

        print(f"\n🔸 With Context Sharing:")
        print(f"   Average Tokens: {scenarios.get('With Context Sharing', {}).get('avg_tokens', 0):.1f}")
        print(f"   Total Tokens: {scenarios.get('With Context Sharing', {}).get('total_tokens', 0)}")
        print(f"   Average Response Time: {scenarios.get('With Context Sharing', {}).get('avg_response_time', 0):.3f}s")

        print(f"\n💡 Performance Improvements:")
        print(f"   Token Efficiency: {improvements.get('token_efficiency', 0):.1f}%")
        print(f"   Token Savings: {improvements.get('token_savings', 0)} tokens")
        print(f"   Response Time Change: {improvements.get('response_time', 0):.1f}%")

        better_scenario = comparison["summary"]["better_scenario"]
        improvement_pct = comparison["summary"]["token_improvement_pct"]

        if better_scenario == "With Context Sharing":
            print(f"\n✅ Context Sharing shows {improvement_pct:.1f}% better token efficiency!")
        else:
            print(f"\n⚠️  Traditional approach shows {improvement_pct:.1f}% better performance")

        print("="*60)


if __name__ == "__main__":
    tester = SingleAgentTester()
    results = tester.run_comparison_test()

    # 保存结果到JSON文件
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    output_file = f"single_agent_test_results_{timestamp}.json"

    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(results, f, ensure_ascii=False, indent=2)

    print(f"\n📁 Results saved to: {output_file}")
