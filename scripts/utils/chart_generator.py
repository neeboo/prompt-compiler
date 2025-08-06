#!/usr/bin/env python3
"""
Chart Generator - 图表生成工具
"""

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np
from typing import List, Dict, Any
from datetime import datetime
import os
from utils.pc_client import ConversationResult

# 设置中文字体和图表样式
plt.rcParams['font.sans-serif'] = ['Arial', 'DejaVu Sans', 'SimHei', 'Arial Unicode MS']
plt.rcParams['axes.unicode_minus'] = False
plt.rcParams['font.family'] = 'sans-serif'

try:
    plt.style.use('seaborn-v0_8')
except:
    pass


class ChartGenerator:
    """图表生成器"""

    def __init__(self, output_dir: str = "pc_node_charts"):
        self.output_dir = output_dir
        os.makedirs(output_dir, exist_ok=True)

    def generate_comparison_chart(
        self,
        scenario_a_results: List[ConversationResult],
        scenario_b_results: List[ConversationResult],
        scenario_a_name: str = "Without Context Sharing",
        scenario_b_name: str = "With Context Sharing",
        title: str = "Performance Comparison",
        filename: str = None
    ) -> str:
        """生成对比图表 - 专注于Token使用效率对比"""

        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"comparison_{timestamp}.png"

        # 创建一个更简洁的2x2布局，专注于有意义的对比
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle(title, fontsize=16, fontweight='bold')

        # 1. Token使用趋势对比
        turns_a = list(range(1, len(scenario_a_results) + 1))
        turns_b = list(range(1, len(scenario_b_results) + 1))
        tokens_a = [r.tokens for r in scenario_a_results]
        tokens_b = [r.tokens for r in scenario_b_results]

        ax1.plot(turns_a, tokens_a, 'o-', label=scenario_a_name, color='#FF6B6B', linewidth=2, markersize=4)
        ax1.plot(turns_b, tokens_b, 'o-', label=scenario_b_name, color='#4ECDC4', linewidth=2, markersize=4)
        ax1.set_xlabel('Conversation Turn')
        ax1.set_ylabel('Tokens per Turn')
        ax1.set_title('Token Usage Trend')
        ax1.legend()
        ax1.grid(True, alpha=0.3)

        # 2. 累积Token对比
        cumulative_a = np.cumsum(tokens_a)
        cumulative_b = np.cumsum(tokens_b)

        ax2.plot(turns_a, cumulative_a, 'o-', label=scenario_a_name, color='#FF6B6B', linewidth=2, markersize=4)
        ax2.plot(turns_b, cumulative_b, 'o-', label=scenario_b_name, color='#4ECDC4', linewidth=2, markersize=4)
        ax2.set_xlabel('Conversation Turn')
        ax2.set_ylabel('Cumulative Tokens')
        ax2.set_title('Cumulative Token Usage')
        ax2.legend()
        ax2.grid(True, alpha=0.3)

        # 添加节省标注
        if len(cumulative_a) > 0 and len(cumulative_b) > 0:
            total_savings = cumulative_a[-1] - cumulative_b[-1]
            savings_percentage = (total_savings / cumulative_a[-1]) * 100 if cumulative_a[-1] > 0 else 0
            ax2.annotate(f'Savings: {total_savings:,} tokens ({savings_percentage:.1f}%)',
                        xy=(0.5, 0.95), xycoords='axes fraction',
                        ha='center', va='top', fontsize=10, fontweight='bold',
                        bbox=dict(boxstyle='round,pad=0.3', facecolor='lightgreen', alpha=0.7))

        # 3. Token效率对比柱状图
        avg_tokens_a = np.mean(tokens_a) if tokens_a else 0
        avg_tokens_b = np.mean(tokens_b) if tokens_b else 0
        total_tokens_a = sum(tokens_a)
        total_tokens_b = sum(tokens_b)

        categories = ['Average per Turn', 'Total Usage']
        values_a = [avg_tokens_a, total_tokens_a]
        values_b = [avg_tokens_b, total_tokens_b]

        x = np.arange(len(categories))
        width = 0.35

        bars_a = ax3.bar(x - width/2, values_a, width, label=scenario_a_name, color='#FF6B6B', alpha=0.8)
        bars_b = ax3.bar(x + width/2, values_b, width, label=scenario_b_name, color='#4ECDC4', alpha=0.8)

        # 添加数值标签
        for bars in [bars_a, bars_b]:
            for bar in bars:
                height = bar.get_height()
                ax3.annotate(f'{height:,.0f}',
                           xy=(bar.get_x() + bar.get_width() / 2, height),
                           xytext=(0, 3),  # 3 points vertical offset
                           textcoords="offset points",
                           ha='center', va='bottom', fontsize=9)

        ax3.set_xlabel('Metrics')
        ax3.set_ylabel('Tokens')
        ax3.set_title('Token Usage Comparison')
        ax3.set_xticks(x)
        ax3.set_xticklabels(categories)
        ax3.legend()
        ax3.grid(True, alpha=0.3, axis='y')

        # 4. 效率提升汇总
        if avg_tokens_a > 0:
            efficiency_improvement = ((avg_tokens_a - avg_tokens_b) / avg_tokens_a) * 100
            cost_savings_ratio = ((total_tokens_a - total_tokens_b) / total_tokens_a) * 100 if total_tokens_a > 0 else 0
        else:
            efficiency_improvement = 0
            cost_savings_ratio = 0

        # 创建效率提升的饼图或其他可视化
        metrics = ['Token Efficiency', 'Cost Savings']
        improvements = [efficiency_improvement, cost_savings_ratio]
        colors_pie = ['#4ECDC4', '#96CEB4']

        wedges, texts, autotexts = ax4.pie([abs(x) for x in improvements], labels=metrics,
                                          colors=colors_pie, autopct='%1.1f%%', startangle=90)
        ax4.set_title('Performance Improvements')

        # 添加改进说明
        improvement_text = f"Token Efficiency: {efficiency_improvement:.1f}%\nCost Savings: {cost_savings_ratio:.1f}%"
        ax4.text(0, -1.3, improvement_text, ha='center', va='center', fontsize=10,
                bbox=dict(boxstyle='round,pad=0.5', facecolor='lightyellow', alpha=0.8))

        plt.tight_layout()

        filepath = os.path.join(self.output_dir, filename)
        plt.savefig(filepath, dpi=300, bbox_inches='tight')
        plt.close()

        return filepath

    def generate_multi_agent_chart(
        self,
        agent_results: Dict[str, List[ConversationResult]],
        title: str = "Multi-Agent Performance Analysis",
        filename: str = None
    ) -> str:
        """生成多智能体分析图表"""

        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"multi_agent_{timestamp}.png"

        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 12))
        fig.suptitle(title, fontsize=16, fontweight='bold')

        colors = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FECA57']

        # 1. 各智能体Token使用分布
        agent_tokens = []
        agent_labels = []

        for i, (agent_id, results) in enumerate(agent_results.items()):
            tokens = [r.tokens for r in results if r.tokens > 0]
            if tokens:
                agent_tokens.append(tokens)
                agent_labels.append(agent_id)

        if agent_tokens:
            bp = ax1.boxplot(agent_tokens, labels=agent_labels, patch_artist=True)
            for patch, color in zip(bp['boxes'], colors[:len(agent_tokens)]):
                patch.set_facecolor(color)
                patch.set_alpha(0.7)

        ax1.set_ylabel('Tokens per Turn')
        ax1.set_title('Token Usage Distribution by Agent')
        ax1.grid(True, alpha=0.3)

        # 2. 时间序列Token使用
        for i, (agent_id, results) in enumerate(agent_results.items()):
            turns = list(range(1, len(results) + 1))
            tokens = [r.tokens for r in results]
            ax2.plot(turns, tokens, 'o-', label=agent_id, color=colors[i % len(colors)], linewidth=2)

        ax2.set_xlabel('Turn Number')
        ax2.set_ylabel('Tokens')
        ax2.set_title('Token Usage Timeline by Agent')
        ax2.legend()
        ax2.grid(True, alpha=0.3)

        # 3. 智能体活跃度
        agent_activity = {agent_id: len(results) for agent_id, results in agent_results.items()}

        ax3.bar(agent_activity.keys(), agent_activity.values(),
                color=colors[:len(agent_activity)], alpha=0.7)
        ax3.set_ylabel('Number of Turns')
        ax3.set_title('Agent Activity Level')
        ax3.grid(True, alpha=0.3)

        # 4. 平均性能指标
        agent_avg_tokens = {}
        agent_avg_times = {}

        for agent_id, results in agent_results.items():
            tokens = [r.tokens for r in results if r.tokens > 0]
            times = [r.response_time for r in results]
            agent_avg_tokens[agent_id] = np.mean(tokens) if tokens else 0
            agent_avg_times[agent_id] = np.mean(times) if times else 0

        x = np.arange(len(agent_avg_tokens))
        width = 0.35

        ax4.bar(x - width/2, list(agent_avg_tokens.values()), width,
                label='Avg Tokens', color='#FF6B6B', alpha=0.7)
        ax4_twin = ax4.twinx()
        ax4_twin.bar(x + width/2, [t * 1000 for t in agent_avg_times.values()], width,
                     label='Avg Time (ms)', color='#4ECDC4', alpha=0.7)

        ax4.set_xlabel('Agents')
        ax4.set_ylabel('Average Tokens', color='#FF6B6B')
        ax4_twin.set_ylabel('Average Time (ms)', color='#4ECDC4')
        ax4.set_title('Average Performance by Agent')
        ax4.set_xticks(x)
        ax4.set_xticklabels(list(agent_avg_tokens.keys()))
        ax4.grid(True, alpha=0.3)

        plt.tight_layout()

        filepath = os.path.join(self.output_dir, filename)
        plt.savefig(filepath, dpi=300, bbox_inches='tight')
        plt.close()

        return filepath
