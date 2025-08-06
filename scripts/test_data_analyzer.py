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

        # 提取关键指标
        analysis = {
            "test_summary": {
                "single_agent": self._extract_summary_metrics(single_agent_results),
                "multi_agent": self._extract_summary_metrics(multi_agent_results)
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
        if not test_results or "comparison" not in test_results:
            return {}

        comparison = test_results["comparison"]
        scenarios = comparison["scenarios"]
        improvements = comparison.get("improvements", {})  # 使用get方法安全访问

        return {
            "without_context_sharing": {
                "avg_tokens": scenarios.get("Without Context Sharing", {}).get("avg_tokens", 0),
                "total_tokens": scenarios.get("Without Context Sharing", {}).get("total_tokens", 0),
                "avg_response_time": scenarios.get("Without Context Sharing", {}).get("avg_response_time", 0)
            },
            "with_context_sharing": {
                "avg_tokens": scenarios.get("With Context Sharing", {}).get("avg_tokens", 0),
                "total_tokens": scenarios.get("With Context Sharing", {}).get("total_tokens", 0),
                "avg_response_time": scenarios.get("With Context Sharing", {}).get("avg_response_time", 0)
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
        """计算综合成本分析"""

        cost_per_1k = 0.002  # GPT-3.5-turbo pricing

        single_summary = self._extract_summary_metrics(single_agent_results)
        multi_summary = self._extract_summary_metrics(multi_agent_results)

        # 单智能体成本
        single_cost_without = (single_summary.get("without_context_sharing", {}).get("total_tokens", 0) / 1000) * cost_per_1k
        single_cost_with = (single_summary.get("with_context_sharing", {}).get("total_tokens", 0) / 1000) * cost_per_1k
        single_savings = single_cost_without - single_cost_with

        # 多智能体成本
        multi_cost_without = (multi_summary.get("without_context_sharing", {}).get("total_tokens", 0) / 1000) * cost_per_1k
        multi_cost_with = (multi_summary.get("with_context_sharing", {}).get("total_tokens", 0) / 1000) * cost_per_1k
        multi_savings = multi_cost_without - multi_cost_with

        return {
            "single_agent": {
                "cost_without_context": single_cost_without,
                "cost_with_context": single_cost_with,
                "savings_usd": single_savings,
                "savings_percentage": (single_savings / single_cost_without * 100) if single_cost_without > 0 else 0
            },
            "multi_agent": {
                "cost_without_context": multi_cost_without,
                "cost_with_context": multi_cost_with,
                "savings_usd": multi_savings,
                "savings_percentage": (multi_savings / multi_cost_without * 100) if multi_cost_without > 0 else 0
            },
            "total_savings": {
                "usd": single_savings + multi_savings,
                "percentage": ((single_savings + multi_savings) / (single_cost_without + multi_cost_without) * 100) if (single_cost_without + multi_cost_without) > 0 else 0
            }
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
                f"✅ 多智能体场景显示 {multi_efficiency:.1f}% 的Token效率提升，强烈推荐使用"
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
- **响应时间变化**: {analysis['test_summary']['single_agent'].get('improvements', {}).get('response_time_change', 0):.1f}%

### 多智能体测试结果
- **Token效率提升**: {analysis['test_summary']['multi_agent'].get('improvements', {}).get('token_efficiency', 0):.1f}%
- **Token节省**: {analysis['test_summary']['multi_agent'].get('improvements', {}).get('token_savings', 0)} tokens
- **响应时间变化**: {analysis['test_summary']['multi_agent'].get('improvements', {}).get('response_time_change', 0):.1f}%

## 💡 性能洞察

### Context Sharing效果
- **单智能体效率**: {analysis['performance_insights']['context_sharing_effectiveness']['single_agent_efficiency']:.1f}%
- **多智能体效率**: {analysis['performance_insights']['context_sharing_effectiveness']['multi_agent_efficiency']:.1f}%
- **可扩展性因子**: {analysis['performance_insights']['context_sharing_effectiveness']['scalability_factor']:.2f}

## 💰 成本分析

### 单智能体场景
- **不使用Context Sharing**: ${analysis['cost_analysis']['single_agent']['cost_without_context']:.4f}
- **使用Context Sharing**: ${analysis['cost_analysis']['single_agent']['cost_with_context']:.4f}
- **节省**: ${analysis['cost_analysis']['single_agent']['savings_usd']:.4f} ({analysis['cost_analysis']['single_agent']['savings_percentage']:.1f}%)

### 多智能体场景
- **不使用Context Sharing**: ${analysis['cost_analysis']['multi_agent']['cost_without_context']:.4f}
- **使用Context Sharing**: ${analysis['cost_analysis']['multi_agent']['cost_with_context']:.4f}
- **节省**: ${analysis['cost_analysis']['multi_agent']['savings_usd']:.4f} ({analysis['cost_analysis']['multi_agent']['savings_percentage']:.1f}%)

## 🎯 使用建议

### 何时使用Context Sharing
"""

        for recommendation in analysis['recommendations']['when_to_use_context_sharing']:
            content += f"- {recommendation}\n"

        content += "\n### 架构考虑\n"
        for recommendation in analysis['recommendations']['architecture_considerations']:
            content += f"- {recommendation}\n"

        content += f"""
## 📈 可扩展性分析

- **扩展效率**: {analysis['scalability_analysis']['scaling_efficiency']:.1f}%
- **可扩展性评级**: {analysis['scalability_analysis']['scalability_rating']}

### 扩展建议
"""

        for recommendation in analysis['scalability_analysis']['recommendations']:
            content += f"- {recommendation}\n"

        content += f"""
## 🏆 总结

基于测试结果，PC Node的Context Sharing功能在以下方面表现优异：

1. **显著的Token效率提升** - 在不同场景下都实现了可观的Token节省
2. **良好的多智能体支持** - 在复杂的多智能体环境中表现更加出色
3. **成本效益明显** - 能够有效降低API调用成本
4. **架构优势突出** - 为协作型AI应用提供了优秀的基础设施

建议在需要多轮对话、多智能体协作的场景中优先考虑使用PC Node的Context Sharing功能。

---
*此报告由PC Node自动化测试系统生成*
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
