#!/usr/bin/env python3
"""
主测试运行器 - 协调所有测试并生成综合报告
"""

import json
import time
import os
import sys
from datetime import datetime
from typing import Dict, Any
from test_single_agent import SingleAgentTester
from test_multi_agent import MultiAgentTester
from test_data_analyzer import TestDataAnalyzer


class TestRunner:
    def __init__(self):
        self.results_dir = "test_results"
        os.makedirs(self.results_dir, exist_ok=True)

        self.single_agent_tester = SingleAgentTester()
        self.multi_agent_tester = MultiAgentTester()
        self.data_analyzer = TestDataAnalyzer()

    def run_all_tests(self, skip_single=False, skip_multi=False) -> Dict[str, Any]:
        """运行所有测试并生成综合报告"""

        print("🚀 Starting PC Node Comprehensive Performance Testing")
        print("=" * 60)

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        results = {
            "test_run_id": f"pc_node_test_{timestamp}",
            "start_time": datetime.now().isoformat(),
            "tests_completed": [],
            "errors": []
        }

        # 运行单智能体测试
        if not skip_single:
            try:
                print("\n🤖 Running Single Agent Tests...")
                single_results = self.single_agent_tester.run_comparison_test()
                results["single_agent_results"] = single_results
                results["tests_completed"].append("single_agent")

                # 保存单独的结果文件
                single_file = os.path.join(self.results_dir, f"single_agent_{timestamp}.json")
                with open(single_file, 'w', encoding='utf-8') as f:
                    json.dump(single_results, f, ensure_ascii=False, indent=2)

                print(f"✅ Single agent test completed - saved to {single_file}")

            except Exception as e:
                error_msg = f"Single agent test failed: {str(e)}"
                print(f"❌ {error_msg}")
                results["errors"].append(error_msg)

        # 运行多智能体测试
        if not skip_multi:
            try:
                print("\n👥 Running Multi-Agent Tests...")
                multi_results = self.multi_agent_tester.run_comparison_test()
                results["multi_agent_results"] = multi_results
                results["tests_completed"].append("multi_agent")

                # 保存单独的结果文件
                multi_file = os.path.join(self.results_dir, f"multi_agent_{timestamp}.json")
                with open(multi_file, 'w', encoding='utf-8') as f:
                    json.dump(multi_results, f, ensure_ascii=False, indent=2)

                print(f"✅ Multi-agent test completed - saved to {multi_file}")

            except Exception as e:
                error_msg = f"Multi-agent test failed: {str(e)}"
                print(f"❌ {error_msg}")
                results["errors"].append(error_msg)

        # 生成综合分析
        if "single_agent" in results["tests_completed"] and "multi_agent" in results["tests_completed"]:
            try:
                print("\n📊 Generating Comprehensive Analysis...")

                # 🔍 调试：打印传递给分析器的���据类型
                print(f"🔍 Single agent test type: {results['single_agent_results'].get('test_type', 'unknown')}")
                print(f"🔍 Multi agent test type: {results['multi_agent_results'].get('test_type', 'unknown')}")

                analysis = self.data_analyzer.analyze_comprehensive_results(
                    results["single_agent_results"],
                    results["multi_agent_results"]
                )
                results["comprehensive_analysis"] = analysis

                # 生成Markdown报告
                report_file = os.path.join(self.results_dir, f"analysis_report_{timestamp}.md")
                self.data_analyzer.generate_markdown_report(analysis, report_file)

                print(f"✅ Comprehensive analysis completed - report saved to {report_file}")

            except Exception as e:
                error_msg = f"Comprehensive analysis failed: {str(e)}"
                print(f"❌ {error_msg}")
                results["errors"].append(error_msg)

        results["end_time"] = datetime.now().isoformat()
        results["total_duration"] = (datetime.fromisoformat(results["end_time"]) -
                                   datetime.fromisoformat(results["start_time"])).total_seconds()

        # 保存完整的测试结果
        complete_results_file = os.path.join(self.results_dir, f"complete_test_results_{timestamp}.json")
        with open(complete_results_file, 'w', encoding='utf-8') as f:
            json.dump(results, f, ensure_ascii=False, indent=2)

        self._print_final_summary(results)

        return results

    def _print_final_summary(self, results: Dict[str, Any]):
        """打印最终测试总结"""

        print("\n" + "="*80)
        print("🏆 PC NODE COMPREHENSIVE TEST SUMMARY")
        print("="*80)

        print(f"📅 Test Run ID: {results['test_run_id']}")
        print(f"⏱️  Total Duration: {results['total_duration']:.2f} seconds")
        print(f"✅ Tests Completed: {', '.join(results['tests_completed'])}")

        if results['errors']:
            print(f"❌ Errors: {len(results['errors'])}")
            for error in results['errors']:
                print(f"   - {error}")

        # 如果有综合分析，显示关键指标
        if "comprehensive_analysis" in results:
            analysis = results["comprehensive_analysis"]

            print("\n📊 KEY PERFORMANCE INDICATORS:")

            if "test_summary" in analysis:
                single_efficiency = analysis["test_summary"]["single_agent"].get("improvements", {}).get("token_efficiency", 0)
                multi_efficiency = analysis["test_summary"]["multi_agent"].get("improvements", {}).get("token_efficiency", 0)

                print(f"🤖 Single Agent Token Efficiency: {single_efficiency:.1f}%")
                print(f"👥 Multi-Agent Token Efficiency: {multi_efficiency:.1f}%")

            if "cost_analysis" in analysis:
                total_savings = analysis["cost_analysis"]["total_savings"]["percentage"]
                print(f"💰 Overall Cost Savings: {total_savings:.1f}%")

            if "scalability_analysis" in analysis:
                scalability_rating = analysis["scalability_analysis"]["scalability_rating"]
                print(f"📈 Scalability Rating: {scalability_rating}")

        print("\n🎯 FINAL VERDICT:")
        if results["tests_completed"] and not results["errors"]:
            if "comprehensive_analysis" in results:
                single_eff = results["comprehensive_analysis"]["test_summary"]["single_agent"].get("improvements", {}).get("token_efficiency", 0)
                multi_eff = results["comprehensive_analysis"]["test_summary"]["multi_agent"].get("improvements", {}).get("token_efficiency", 0)

                if single_eff > 20 and multi_eff > 20:
                    print("🌟 EXCELLENT: PC Node Context Sharing shows outstanding performance!")
                    print("   Recommended for production use in both single and multi-agent scenarios.")
                elif single_eff > 10 or multi_eff > 10:
                    print("✅ GOOD: PC Node Context Sharing demonstrates clear benefits.")
                    print("   Recommended for scenarios with moderate to high conversation complexity.")
                else:
                    print("⚠️  MIXED: PC Node Context Sharing shows some benefits but needs optimization.")
            else:
                print("✅ Tests completed successfully but comprehensive analysis unavailable.")
        else:
            print("❌ INCOMPLETE: Some tests failed or were skipped. Review errors above.")

        print("="*80)
        print(f"📁 All results saved in: {self.results_dir}/")
        print("="*80)


def main():
    """主函数，支持命令行参数"""

    import argparse

    parser = argparse.ArgumentParser(description="PC Node Performance Test Runner")
    parser.add_argument("--skip-single", action="store_true",
                       help="Skip single agent tests")
    parser.add_argument("--skip-multi", action="store_true",
                       help="Skip multi-agent tests")
    parser.add_argument("--quick", action="store_true",
                       help="Run quick tests (reduced conversation rounds)")

    args = parser.parse_args()

    if args.quick:
        print("⚡ Quick test mode enabled - using reduced conversation rounds")
        # 可以在这里修改配置以减少测试轮数

    runner = TestRunner()

    try:
        results = runner.run_all_tests(
            skip_single=args.skip_single,
            skip_multi=args.skip_multi
        )

        # 根据测试结果确定退出代码
        if results["errors"]:
            sys.exit(1)  # 有错误时退出代码为1
        else:
            sys.exit(0)  # 成功时退出代码为0

    except KeyboardInterrupt:
        print("\n⚠️  Test interrupted by user")
        sys.exit(130)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
