#!/usr/bin/env python3
"""
多智能体多轮对话测试
比较开启和关闭Context Sharing在多智能体场景下的性能差异
"""

import json
import time
from typing import List, Dict, Any
from utils.pc_client import PCNodeClient, ConversationResult
from utils.performance_metrics import MetricsCalculator
from utils.chart_generator import ChartGenerator


class MultiAgentTester:
    def __init__(self, config_path: str = "configs/multi_agent_config.json"):
        with open(config_path, 'r', encoding='utf-8') as f:
            self.config = json.load(f)

        self.client = PCNodeClient(self.config['test_config']['base_url'])
        self.chart_generator = ChartGenerator()

    def test_without_context_sharing(self) -> Dict[str, List[ConversationResult]]:
        """测试不开启Context Sharing的多智能体场景"""
        print("🔍 Testing Multi-Agent WITHOUT Context Sharing...")

        conversation = self.config['conversation_scenarios']['enterprise_project']
        agent_results = {}

        # 记录所有对话历史，每个智能体都需要完整历史
        full_conversation_history = []

        for i, turn in enumerate(conversation):
            agent_id = turn['agent']
            message = turn['message']

            print(f"   Turn {i+1}/{len(conversation)} - {self.config['agents'][agent_id]['name']}: {message[:50]}...")

            # 每次都传递完整的对话历史
            full_conversation_history.append({"role": "user", "content": message})

            result = self.client.chat_completion(
                messages=full_conversation_history.copy(),
                context_sharing=False,
                model=self.config['test_config']['model'],
                temperature=self.config['test_config']['temperature'],
                max_tokens=self.config['test_config']['max_tokens']
            )

            # 记录每个智能体的结果
            if agent_id not in agent_results:
                agent_results[agent_id] = []
            agent_results[agent_id].append(result)

            # 将回复添加到历史中
            if result.content:
                full_conversation_history.append({"role": "assistant", "content": result.content})

            time.sleep(0.1)

        return agent_results

    def test_with_context_sharing(self) -> Dict[str, List[ConversationResult]]:
        """测试开启Context Sharing的多智能体场景"""
        print("🔍 Testing Multi-Agent WITH Context Sharing...")

        conversation = self.config['conversation_scenarios']['enterprise_project']
        agent_results = {}

        for i, turn in enumerate(conversation):
            agent_id = turn['agent']
            message = turn['message']

            print(f"   Turn {i+1}/{len(conversation)} - {self.config['agents'][agent_id]['name']}: {message[:50]}...")

            # 使用context sharing，只需要发送当前消息
            result = self.client.chat_completion(
                messages=[{"role": "user", "content": message}],
                agent_id=agent_id,
                context_sharing=True,
                model=self.config['test_config']['model'],
                temperature=self.config['test_config']['temperature'],
                max_tokens=self.config['test_config']['max_tokens']
            )

            # 记录每个智能体的结果
            if agent_id not in agent_results:
                agent_results[agent_id] = []
            agent_results[agent_id].append(result)

            time.sleep(0.1)

        return agent_results

    def run_comparison_test(self) -> Dict[str, Any]:
        """运行多智能体对比测试"""
        print("🚀 Starting Multi-Agent Comparison Test...")

        # 健康检查
        if not self.client.health_check():
            print("❌ PC Node is not healthy, aborting test")
            return {}

        # 运行两个场景
        start_time = time.time()

        results_without = self.test_without_context_sharing()
        results_with = self.test_with_context_sharing()

        total_time = time.time() - start_time

        # 将多智能体结果合并为单一列表进行对比
        all_results_without = []
        all_results_with = []

        for agent_id in results_without:
            all_results_without.extend(results_without[agent_id])

        for agent_id in results_with:
            all_results_with.extend(results_with[agent_id])

        # 计算整体性能指标对比
        overall_comparison = MetricsCalculator.compare_scenarios(
            all_results_without,
            all_results_with,
            "Without Context Sharing",
            "With Context Sharing"
        )

        # 只生成主要的对比图表，移除响应时间维度
        overall_chart_path = self.chart_generator.generate_comparison_chart(
            all_results_without,
            all_results_with,
            "Without Context Sharing",
            "With Context Sharing",
            "Multi-Agent Performance Comparison (Token Efficiency Focus)",
            "multi_agent_comparison.png"
        )

        # 计算每个智能体的性能指标
        agent_metrics = {}
        for agent_id in self.config['agents']:
            agent_name = self.config['agents'][agent_id]['name']

            if agent_id in results_without and agent_id in results_with:
                agent_comparison = MetricsCalculator.compare_scenarios(
                    results_without[agent_id],
                    results_with[agent_id],
                    f"{agent_name} (Without)",
                    f"{agent_name} (With)"
                )
                agent_metrics[agent_id] = {
                    "name": agent_name,
                    "comparison": agent_comparison,
                    "turns_without": len(results_without[agent_id]),
                    "turns_with": len(results_with[agent_id])
                }

        # 准备返回结果
        result = {
            "test_type": "multi_agent_comparison",
            "total_test_time": total_time,
            "total_conversation_turns": len(self.config['conversation_scenarios']['enterprise_project']),
            "participating_agents": len(self.config['agents']),
            "overall_comparison": overall_comparison,
            "agent_specific_metrics": agent_metrics,
            "chart_paths": {
                "overall_comparison": overall_chart_path
            },
            "raw_results": {
                "without_context_sharing": {
                    agent_id: [
                        {
                            "turn": i+1,
                            "tokens": r.tokens,
                            "response_time": r.response_time,
                            "compression_ratio": r.compression_ratio,
                            "context_size": r.context_size
                        }
                        for i, r in enumerate(results)
                    ]
                    for agent_id, results in results_without.items()
                },
                "with_context_sharing": {
                    agent_id: [
                        {
                            "turn": i+1,
                            "tokens": r.tokens,
                            "response_time": r.response_time,
                            "compression_ratio": r.compression_ratio,
                            "context_size": r.context_size
                        }
                        for i, r in enumerate(results)
                    ]
                    for agent_id, results in results_with.items()
                }
            }
        }

        self._print_summary(overall_comparison, agent_metrics)

        return result

    def _print_summary(self, overall_comparison: Dict[str, Any], agent_metrics: Dict[str, Any]):
        """打印测试总结"""
        print("\n" + "="*70)
        print("📊 MULTI-AGENT TEST SUMMARY")
        print("="*70)

        # 整体性能对比
        scenarios = overall_comparison["scenarios"]
        improvements = overall_comparison["improvements"]

        print(f"🔸 Overall Performance (Without Context Sharing):")
        print(f"   Average Tokens: {scenarios['Without Context Sharing']['avg_tokens']:.1f}")
        print(f"   Total Tokens: {scenarios['Without Context Sharing']['total_tokens']}")
        print(f"   Average Response Time: {scenarios['Without Context Sharing']['avg_response_time']:.3f}s")

        print(f"\n🔸 Overall Performance (With Context Sharing):")
        print(f"   Average Tokens: {scenarios['With Context Sharing']['avg_tokens']:.1f}")
        print(f"   Total Tokens: {scenarios['With Context Sharing']['total_tokens']}")
        print(f"   Average Response Time: {scenarios['With Context Sharing']['avg_response_time']:.3f}s")

        print(f"\n💡 Overall Improvements:")
        print(f"   Token Efficiency: {improvements['token_efficiency']:.1f}%")
        print(f"   Token Savings: {improvements['token_savings']} tokens")
        print(f"   Response Time Change: {improvements['response_time']:.1f}%")

        # 各智能体详细表现
        print(f"\n👥 Agent-Specific Performance:")
        for agent_id, metrics in agent_metrics.items():
            agent_improvements = metrics['comparison']['improvements']
            print(f"   📋 {metrics['name']}:")
            print(f"      Token Efficiency: {agent_improvements['token_efficiency']:.1f}%")
            print(f"      Token Savings: {agent_improvements['token_savings']} tokens")
            print(f"      Turns: {metrics['turns_with']}")

        better_scenario = overall_comparison["summary"]["better_scenario"]
        improvement_pct = overall_comparison["summary"]["token_improvement_pct"]

        if better_scenario == "With Context Sharing":
            print(f"\n✅ Multi-Agent Context Sharing shows {improvement_pct:.1f}% better token efficiency!")
            print("🤝 Successful cross-agent knowledge sharing demonstrated!")
        else:
            print(f"\n⚠️  Traditional approach shows {improvement_pct:.1f}% better performance")

        print("="*70)


if __name__ == "__main__":
    tester = MultiAgentTester()
    results = tester.run_comparison_test()

    # 保存结果到JSON文件
    timestamp = time.strftime("%Y%m%d_%H%M%S")
    output_file = f"multi_agent_test_results_{timestamp}.json"

    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(results, f, ensure_ascii=False, indent=2)

    print(f"\n📁 Results saved to: {output_file}")
