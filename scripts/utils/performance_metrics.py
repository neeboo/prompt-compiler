#!/usr/bin/env python3
"""
Performance Metrics - 性能指标计算工具
"""

from typing import List, Dict, Any, Tuple
import numpy as np
from dataclasses import dataclass
from utils.pc_client import ConversationResult


@dataclass
class PerformanceMetrics:
    """性能指标数据类"""
    avg_tokens: float
    total_tokens: int
    avg_response_time: float
    token_growth_rate: float
    compression_efficiency: float
    stability_coefficient: float  # 变异系数


class MetricsCalculator:
    """性能指标计算器"""

    @staticmethod
    def calculate_metrics(results: List[ConversationResult]) -> PerformanceMetrics:
        """计算基础性能指标"""
        if not results:
            return PerformanceMetrics(0, 0, 0, 0, 0, 0)

        tokens = [r.tokens for r in results if r.tokens > 0]
        response_times = [r.response_time for r in results]

        # 基础统计
        avg_tokens = np.mean(tokens) if tokens else 0
        total_tokens = sum(tokens)
        avg_response_time = np.mean(response_times)

        # Token增长率计算
        token_growth_rate = 0
        if len(tokens) > 1:
            first_half = tokens[:len(tokens)//2]
            second_half = tokens[len(tokens)//2:]
            if first_half and second_half:
                growth = (np.mean(second_half) - np.mean(first_half)) / np.mean(first_half)
                token_growth_rate = growth * 100

        # 压缩效率
        compression_ratios = [r.compression_ratio for r in results if r.compression_ratio is not None]
        compression_efficiency = np.mean(compression_ratios) if compression_ratios else 0

        # 稳定性系数（变异系数）
        stability_coefficient = (np.std(tokens) / avg_tokens * 100) if avg_tokens > 0 else 0

        return PerformanceMetrics(
            avg_tokens=avg_tokens,
            total_tokens=total_tokens,
            avg_response_time=avg_response_time,
            token_growth_rate=token_growth_rate,
            compression_efficiency=compression_efficiency,
            stability_coefficient=stability_coefficient
        )

    @staticmethod
    def compare_scenarios(
        scenario_a: List[ConversationResult],
        scenario_b: List[ConversationResult],
        scenario_a_name: str = "Scenario A",
        scenario_b_name: str = "Scenario B"
    ) -> Dict[str, Any]:
        """比较两个测试场景"""

        metrics_a = MetricsCalculator.calculate_metrics(scenario_a)
        metrics_b = MetricsCalculator.calculate_metrics(scenario_b)

        # 计算改进百分比
        token_improvement = ((metrics_a.avg_tokens - metrics_b.avg_tokens) / metrics_a.avg_tokens * 100) if metrics_a.avg_tokens > 0 else 0
        time_improvement = ((metrics_a.avg_response_time - metrics_b.avg_response_time) / metrics_a.avg_response_time * 100) if metrics_a.avg_response_time > 0 else 0

        return {
            "scenarios": {
                scenario_a_name: {
                    "avg_tokens": metrics_a.avg_tokens,
                    "total_tokens": metrics_a.total_tokens,
                    "avg_response_time": metrics_a.avg_response_time,
                    "token_growth_rate": metrics_a.token_growth_rate,
                    "compression_efficiency": metrics_a.compression_efficiency,
                    "stability_coefficient": metrics_a.stability_coefficient
                },
                scenario_b_name: {
                    "avg_tokens": metrics_b.avg_tokens,
                    "total_tokens": metrics_b.total_tokens,
                    "avg_response_time": metrics_b.avg_response_time,
                    "token_growth_rate": metrics_b.token_growth_rate,
                    "compression_efficiency": metrics_b.compression_efficiency,
                    "stability_coefficient": metrics_b.stability_coefficient
                }
            },
            "improvements": {
                "token_efficiency": token_improvement,
                "response_time": time_improvement,
                "token_savings": metrics_a.total_tokens - metrics_b.total_tokens
            },
            "summary": {
                "better_scenario": scenario_b_name if token_improvement > 0 else scenario_a_name,
                "token_improvement_pct": abs(token_improvement),
                "time_improvement_pct": abs(time_improvement)
            }
        }

    @staticmethod
    def calculate_cost_analysis(
        results: List[ConversationResult],
        cost_per_1k_tokens: float = 0.002  # GPT-3.5-turbo pricing
    ) -> Dict[str, float]:
        """计算成本分析"""
        total_tokens = sum(r.tokens for r in results if r.tokens > 0)
        estimated_cost = (total_tokens / 1000) * cost_per_1k_tokens

        return {
            "total_tokens": total_tokens,
            "estimated_cost_usd": estimated_cost,
            "cost_per_turn": estimated_cost / len(results) if results else 0,
            "tokens_per_turn": total_tokens / len(results) if results else 0
        }
