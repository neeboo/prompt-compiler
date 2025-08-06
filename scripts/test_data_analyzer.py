#!/usr/bin/env python3
"""
数据分析器 - 综合分析测试结果并生成报告
"""

import json
import time
from typing import List, Dict, Any
import numpy as np
from datetime import datetime
from utils.performance_metrics import MetricsCalculator
from utils.chart_generator import ChartGenerator


class TestDataAnalyzer:
    def __init__(self):
        self.chart_generator = ChartGenerator()

    def analyze_comprehensive_results(
        self,
        single_agent_results: Dict[str, Any],
        multi_agent_results: Dict[str, Any]
    ) -> Dict[str, Any]:
        """综合分析单智能体和多智能体测试结果"""

        print("📊 Analyzing comprehensive test results...")

        # 🔍 添加调试：确认传入的参数类型
        print(f"🔍 Received single_agent type: {single_agent_results.get('test_type', 'unknown')}")
        print(f"🔍 Received multi_agent type: {multi_agent_results.get('test_type', 'unknown')}")

        # 提取关键指标
        print("🔍 Extracting single agent summary...")
        single_agent_summary = self._extract_summary_metrics(single_agent_results)

        print("🔍 Extracting multi agent summary...")
        multi_agent_summary = self._extract_summary_metrics(multi_agent_results)

        analysis = {
            "test_summary": {
                "single_agent": single_agent_summary,
                "multi_agent": multi_agent_summary
            },
            "performance_insights": self._generate_performance_insights(
                single_agent_results, multi_agent_results
            ),
            "cost_analysis": self._calculate_comprehensive_cost_analysis(
                single_agent_results, multi_agent_results
            ),
            "recommendations": self._generate_recommendations(
                single_agent_results, multi_agent_results
            ),
            "scalability_analysis": self._analyze_scalability(
                single_agent_results, multi_agent_results
            )
        }

        # 生成综合对比图表
        chart_path = self._generate_comprehensive_chart(
            single_agent_results, multi_agent_results
        )
        analysis["comprehensive_chart"] = chart_path

        return analysis

    def _extract_summary_metrics(self, test_results: Dict[str, Any]) -> Dict[str, Any]:
        """提取测试结果的关键指标"""
        if not test_results:
            return {
                "without_context_sharing": {"avg_tokens": 0, "total_tokens": 0, "avg_response_time": 0},
                "with_context_sharing": {"avg_tokens": 0, "total_tokens": 0, "avg_response_time": 0},
                "improvements": {"token_efficiency": 0, "token_savings": 0, "response_time_change": 0}
            }

        # 🔍 处理不同类型的测试结果数据结构
        test_type = test_results.get("test_type", "unknown")
        print(f"🔍 Processing {test_type}")

        # 根据测试类型选择正确的数据字段
        if test_type == "multi_agent_comparison":
            # 多智能体使用 overall_comparison 字段
            comparison = test_results.get("overall_comparison", {})
        else:
            # 单智能体使用 comparison 字段
            comparison = test_results.get("comparison", {})

        if not comparison:
            return {
                "without_context_sharing": {"avg_tokens": 0, "total_tokens": 0, "avg_response_time": 0},
                "with_context_sharing": {"avg_tokens": 0, "total_tokens": 0, "avg_response_time": 0},
                "improvements": {"token_efficiency": 0, "token_savings": 0, "response_time_change": 0}
            }

        scenarios = comparison.get("scenarios", {})
        improvements = comparison.get("improvements", {})

        print(f"🔍 {test_type} - scenarios keys: {list(scenarios.keys())}")
        print(f"🔍 {test_type} - token_efficiency: {improvements.get('token_efficiency', 0):.1f}%")

        # 提取场景数据
        without_data = scenarios.get("Without Context Sharing", {})
        with_data = scenarios.get("With Context Sharing", {})

        return {
            "without_context_sharing": {
                "avg_tokens": without_data.get("avg_tokens", 0),
                "total_tokens": without_data.get("total_tokens", 0),
                "avg_response_time": without_data.get("avg_response_time", 0)
            },
            "with_context_sharing": {
                "avg_tokens": with_data.get("avg_tokens", 0),
                "total_tokens": with_data.get("total_tokens", 0),
                "avg_response_time": with_data.get("avg_response_time", 0)
            },
            "improvements": {
                "token_efficiency": improvements.get("token_efficiency", 0),
                "token_savings": improvements.get("token_savings", 0),
                "response_time_change": improvements.get("response_time", 0)
            }
        }

    def _generate_performance_insights(
        self,
        single_agent_results: Dict[str, Any],
        multi_agent_results: Dict[str, Any]
    ) -> Dict[str, Any]:
        """生成性能洞察"""

        single_summary = self._extract_summary_metrics(single_agent_results)
        multi_summary = self._extract_summary_metrics(multi_agent_results)

        insights = {
            "context_sharing_effectiveness": {
                "single_agent_efficiency": single_summary.get("improvements", {}).get("token_efficiency", 0),
                "multi_agent_efficiency": multi_summary.get("improvements", {}).get("token_efficiency", 0),
                "scalability_factor": 0
            },
            "complexity_impact": {
                "single_agent_avg_tokens": single_summary.get("with_context_sharing", {}).get("avg_tokens", 0),
                "multi_agent_avg_tokens": multi_summary.get("with_context_sharing", {}).get("avg_tokens", 0),
                "complexity_overhead": 0
            }
        }

        # 计算可扩展性因子
        single_efficiency = insights["context_sharing_effectiveness"]["single_agent_efficiency"]
        multi_efficiency = insights["context_sharing_effectiveness"]["multi_agent_efficiency"]

        if single_efficiency > 0:
            insights["context_sharing_effectiveness"]["scalability_factor"] = multi_efficiency / single_efficiency

        # 计算复杂度开销
        single_tokens = insights["complexity_impact"]["single_agent_avg_tokens"]
        multi_tokens = insights["complexity_impact"]["multi_agent_avg_tokens"]

        if single_tokens > 0:
            insights["complexity_impact"]["complexity_overhead"] = (multi_tokens - single_tokens) / single_tokens * 100

        return insights

    def _calculate_comprehensive_cost_analysis(
        self,
        single_agent_results: Dict[str, Any],
        multi_agent_results: Dict[str, Any]
    ) -> Dict[str, Any]:
        """计算综合成本分析 - 基于Token节省量"""

        single_summary = self._extract_summary_metrics(single_agent_results)
        multi_summary = self._extract_summary_metrics(multi_agent_results)

        # 单智能体Token节省
        single_tokens_without = single_summary.get("without_context_sharing", {}).get("total_tokens", 0)
        single_tokens_with = single_summary.get("with_context_sharing", {}).get("total_tokens", 0)
        single_token_savings = single_tokens_without - single_tokens_with

        # 多智能体Token节省
        multi_tokens_without = multi_summary.get("without_context_sharing", {}).get("total_tokens", 0)
        multi_tokens_with = multi_summary.get("with_context_sharing", {}).get("total_tokens", 0)
        multi_token_savings = multi_tokens_without - multi_tokens_with

        # 计算每轮Token节省
        single_rounds = 19  # 单智能体测试轮数
        multi_rounds = 20   # 多智能体测试轮数

        single_per_round_token_savings = single_token_savings / single_rounds if single_rounds > 0 else 0
        multi_per_round_token_savings = multi_token_savings / multi_rounds if multi_rounds > 0 else 0
        average_per_round_token_savings = (single_per_round_token_savings + multi_per_round_token_savings) / 2

        return {
            "single_agent": {
                "tokens_without_context": single_tokens_without,
                "tokens_with_context": single_tokens_with,
                "token_savings": single_token_savings,
                "savings_percentage": (single_token_savings / single_tokens_without * 100) if single_tokens_without > 0 else 0,
                "per_round_token_savings": single_per_round_token_savings
            },
            "multi_agent": {
                "tokens_without_context": multi_tokens_without,
                "tokens_with_context": multi_tokens_with,
                "token_savings": multi_token_savings,
                "savings_percentage": (multi_token_savings / multi_tokens_without * 100) if multi_tokens_without > 0 else 0,
                "per_round_token_savings": multi_per_round_token_savings
            },
            "total_savings": {
                "tokens": single_token_savings + multi_token_savings,
                "percentage": ((single_token_savings + multi_token_savings) / (single_tokens_without + multi_tokens_without) * 100) if (single_tokens_without + multi_tokens_without) > 0 else 0
            },
            "average_per_round_token_savings": average_per_round_token_savings
        }

    def _generate_recommendations(
        self,
        single_agent_results: Dict[str, Any],
        multi_agent_results: Dict[str, Any]
    ) -> Dict[str, List[str]]:
        """生成使用建议"""

        single_summary = self._extract_summary_metrics(single_agent_results)
        multi_summary = self._extract_summary_metrics(multi_agent_results)

        recommendations = {
            "when_to_use_context_sharing": [],
            "performance_optimization": [],
            "cost_optimization": [],
            "architecture_considerations": []
        }

        # 基于性能数据生成建议
        single_efficiency = single_summary.get("improvements", {}).get("token_efficiency", 0)
        multi_efficiency = multi_summary.get("improvements", {}).get("token_efficiency", 0)

        if single_efficiency > 20:
            recommendations["when_to_use_context_sharing"].append(
                f"✅ 单智能体场景显示 {single_efficiency:.1f}% 的Token效率提升，推荐使用"
            )

        if multi_efficiency > 30:
            recommendations["when_to_use_context_sharing"].append(
                f"✅ 多智能体场景显示 {multi_efficiency:.1f}% 的Token效率提升���强烈推荐使用"
            )

        if multi_efficiency > single_efficiency * 1.2:
            recommendations["architecture_considerations"].append(
                "🏗️  Context Sharing在多智能体环境中表现更优，适合协作型应用"
            )

        # 性能优化建议
        if single_efficiency > 0:
            recommendations["performance_optimization"].append(
                "⚡ Context Sharing有效减少Token使用，提升响应效率"
            )

        # 成本优化建议
        recommendations["cost_optimization"].append(
            "💰 通过Context Sharing可显著降低API调用成本"
        )

        return recommendations

    def _analyze_scalability(
        self,
        single_agent_results: Dict[str, Any],
        multi_agent_results: Dict[str, Any]
    ) -> Dict[str, Any]:
        """分析可扩展性"""

        single_summary = self._extract_summary_metrics(single_agent_results)
        multi_summary = self._extract_summary_metrics(multi_agent_results)

        # 计算扩展效率
        single_tokens = single_summary.get("with_context_sharing", {}).get("avg_tokens", 0)
        multi_tokens = multi_summary.get("with_context_sharing", {}).get("avg_tokens", 0)

        scalability_efficiency = 0
        if single_tokens > 0:
            scalability_efficiency = (1 - (multi_tokens / single_tokens)) * 100

        return {
            "scaling_efficiency": scalability_efficiency,
            "single_agent_baseline": single_tokens,
            "multi_agent_performance": multi_tokens,
            "scalability_rating": self._rate_scalability(scalability_efficiency),
            "recommendations": self._scalability_recommendations(scalability_efficiency)
        }

    def _rate_scalability(self, efficiency: float) -> str:
        """评级可扩展性"""
        if efficiency >= 20:
            return "⭐⭐⭐⭐⭐ 优秀"
        elif efficiency >= 10:
            return "⭐⭐⭐⭐ 良好"
        elif efficiency >= 0:
            return "⭐⭐⭐ 一般"
        else:
            return "⭐⭐ 需要优化"

    def _scalability_recommendations(self, efficiency: float) -> List[str]:
        """可扩展性建议"""
        if efficiency >= 15:
            return ["🚀 Context Sharing展现出色的扩展性能，适合大规模部署"]
        elif efficiency >= 5:
            return ["📈 Context Sharing具备良好扩展潜力，建议在复杂场景中使用"]
        else:
            return ["🔧 建议进一步优化Context Sharing算法以提升扩展效率"]

    def _generate_comprehensive_chart(
        self,
        single_agent_results: Dict[str, Any],
        multi_agent_results: Dict[str, Any]
    ) -> str:
        """生成综合对比图表"""

        # 这里可以创建一个综合的仪表板图表
        # 暂时返回路径，实际实现需要更复杂的可视化
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        return f"comprehensive_analysis_{timestamp}.png"

    def generate_markdown_report(
        self,
        analysis: Dict[str, Any],
        output_file: str = None
    ) -> str:
        """生成Markdown格式的分析报告"""

        if output_file is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            output_file = f"pc_node_analysis_report_{timestamp}.md"

        report_content = self._build_markdown_content(analysis)

        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(report_content)

        return output_file

    def _build_markdown_content(self, analysis: Dict[str, Any]) -> str:
        """构建Markdown报告内容"""

        timestamp = datetime.now().strftime("%Y年%m月%d日")

        content = f"""# PC Node 性能分析报告

*生成时间: {timestamp}*

## 📊 测试概览

### 单智能体测试结果
- **Token效率提升**: {analysis['test_summary']['single_agent'].get('improvements', {}).get('token_efficiency', 0):.1f}%
- **Token节省**: {analysis['test_summary']['single_agent'].get('improvements', {}).get('token_savings', 0)} tokens

![单智能体性能对比](images/single_agent_comparison.png)

### 多智能体测试结果
- **Token效率提升**: {analysis['test_summary']['multi_agent'].get('improvements', {}).get('token_efficiency', 0):.1f}%
- **Token节省**: {analysis['test_summary']['multi_agent'].get('improvements', {}).get('token_savings', 0)} tokens

![多智能体性能对比](images/multi_agent_comparison.png)

## 💡 性能洞察

### Context Sharing效果
- **单智能体效率**: {analysis['performance_insights']['context_sharing_effectiveness']['single_agent_efficiency']:.1f}%
- **多智能体效率**: {analysis['performance_insights']['context_sharing_effectiveness']['multi_agent_efficiency']:.1f}%
- **可扩展性因子**: {analysis['performance_insights']['context_sharing_effectiveness']['scalability_factor']:.2f}

### 复杂度影响
- **单智能体平均Token**: {analysis['performance_insights']['complexity_impact']['single_agent_avg_tokens']:.0f} tokens
- **多智能体平均Token**: {analysis['performance_insights']['complexity_impact']['multi_agent_avg_tokens']:.0f} tokens
- **复杂度开销**: {analysis['performance_insights']['complexity_impact']['complexity_overhead']:.1f}%

## 💰 Token节省分析

### 单智能体场景
- **不使用Context Sharing**: {analysis['cost_analysis']['single_agent']['tokens_without_context']:,.0f} tokens
- **使用Context Sharing**: {analysis['cost_analysis']['single_agent']['tokens_with_context']:,.0f} tokens
- **节省**: {analysis['cost_analysis']['single_agent']['token_savings']:,.0f} tokens ({analysis['cost_analysis']['single_agent']['savings_percentage']:.1f}%)
- **每轮节省**: {analysis['cost_analysis']['single_agent']['per_round_token_savings']:.0f} tokens

### 多智能体场景
- **不使用Context Sharing**: {analysis['cost_analysis']['multi_agent']['tokens_without_context']:,.0f} tokens
- **使用Context Sharing**: {analysis['cost_analysis']['multi_agent']['tokens_with_context']:,.0f} tokens
- **节省**: {analysis['cost_analysis']['multi_agent']['token_savings']:,.0f} tokens ({analysis['cost_analysis']['multi_agent']['savings_percentage']:.1f}%)
- **每轮节省**: {analysis['cost_analysis']['multi_agent']['per_round_token_savings']:.0f} tokens

### 总体节省
- **总Token节省**: {analysis['cost_analysis']['total_savings']['tokens']:,.0f} tokens
- **总节省比例**: {analysis['cost_analysis']['total_savings']['percentage']:.1f}%
- **平均每轮节省**: {analysis['cost_analysis']['average_per_round_token_savings']:.0f} tokens

## 🎯 使用建议

### 何时使用Context Sharing
"""

        # 添加建议内容
        for recommendation in analysis['recommendations']['when_to_use_context_sharing']:
            content += f"- {recommendation}\n"

        content += "\n### 性能优化建议\n"
        for recommendation in analysis['recommendations']['performance_optimization']:
            content += f"- {recommendation}\n"

        content += "\n### 成本优化建议\n"
        for recommendation in analysis['recommendations']['cost_optimization']:
            content += f"- {recommendation}\n"

        content += "\n### 架构考虑\n"
        for recommendation in analysis['recommendations']['architecture_considerations']:
            content += f"- {recommendation}\n"

        # 添加总结，使用tokens表示
        content += f"""
## 📋 总结

本次测试验证了PC Node在Context Sharing方面的性能表现：

1. **单智能体场景**: Context Sharing带来了{analysis['test_summary']['single_agent'].get('improvements', {}).get('token_efficiency', 0):.1f}%的Token效率提升
2. **多智能体场景**: Context Sharing带来了{analysis['test_summary']['multi_agent'].get('improvements', {}).get('token_efficiency', 0):.1f}%的Token效率提升
3. **Token节省**: 平均每轮对话节省{analysis['cost_analysis']['average_per_round_token_savings']:.0f} tokens
4. **规模效应**: 每1000轮对话节省{analysis['cost_analysis']['average_per_round_token_savings'] * 1000:.0f} tokens

---
*报告由PC Node自动生成 | 数据来源: 综合性能测试*
"""

        return content


if __name__ == "__main__":
    # 示例用法
    analyzer = TestDataAnalyzer()

    # 这里应该加载实际的测试结果
    # single_results = json.load(open("single_agent_test_results.json"))
    # multi_results = json.load(open("multi_agent_test_results.json"))

    # analysis = analyzer.analyze_comprehensive_results(single_results, multi_results)
    # report_file = analyzer.generate_markdown_report(analysis)

    print("📊 Data analyzer ready for comprehensive analysis")
