#!/usr/bin/env python3
"""
PC Node 性能报告生成脚本
自动运行测试并生成完整的性能分析报告
"""

import os
import sys
import json
import shutil
from datetime import datetime
from test_runner import TestRunner
from test_data_analyzer import TestDataAnalyzer


def create_docs_structure():
    """创建docs目录结构"""
    docs_dir = "../docs"
    images_dir = os.path.join(docs_dir, "images")

    # 确保目录存在
    os.makedirs(images_dir, exist_ok=True)
    return docs_dir, images_dir


def move_charts_to_docs(test_results_dir, images_dir):
    """将图表文件移动到docs/images目录"""
    chart_files = []

    # 查找所有图表文件
    for file in os.listdir(test_results_dir):
        if file.endswith('.png'):
            src_path = os.path.join(test_results_dir, file)
            dst_path = os.path.join(images_dir, file)
            shutil.copy2(src_path, dst_path)
            chart_files.append(file)
            print(f"📊 图表已移动: {file} -> docs/images/")

    return chart_files


def generate_english_report(chinese_report_path, docs_dir):
    """生成英文版本的报告"""
    english_report_path = os.path.join(docs_dir, "pc_node_performance_report.md")

    # 读取中文报告
    with open(chinese_report_path, 'r', encoding='utf-8') as f:
        chinese_content = f.read()

    # 更完善的中英文对照翻译
    english_content = chinese_content.replace(
        "# PC Node 性能分析报告", "# PC Node Performance Analysis Report"
    ).replace(
        "*生成时间:", "*Generated on:"
    ).replace(
        "年", "/"
    ).replace(
        "月", "/"
    ).replace(
        "日*", "*"
    ).replace(
        "## 📊 测试概览", "## 📊 Test Overview"
    ).replace(
        "### 单智能体测试结果", "### Single Agent Test Results"
    ).replace(
        "### 多智能体测试结果", "### Multi-Agent Test Results"
    ).replace(
        "**Token效率提升**", "**Token Efficiency Improvement**"
    ).replace(
        "![单智能体性能对比](images/single_agent_comparison.png)",
        "![Single Agent Performance Comparison](images/single_agent_comparison.png)"
    ).replace(
        "![多智能体性能对比](images/multi_agent_comparison.png)",
        "![Multi-Agent Performance Comparison](images/multi_agent_comparison.png)"
    ).replace(
        "## 💡 性能洞察", "## 💡 Performance Insights"
    ).replace(
        "### Context Sharing效果", "### Context Sharing Effectiveness"
    ).replace(
        "**单智能体效率**", "**Single Agent Efficiency**"
    ).replace(
        "**多智能体效率**", "**Multi-Agent Efficiency**"
    ).replace(
        "**可扩展性因子**", "**Scalability Factor**"
    ).replace(
        "### 复杂度影响", "### Complexity Impact"
    ).replace(
        "**单智能体平均Token**", "**Single Agent Avg Tokens**"
    ).replace(
        "**多智能体平均Token**", "**Multi-Agent Avg Tokens**"
    ).replace(
        "**复杂度开销**", "**Complexity Overhead**"
    ).replace(
        "## 💰 成本分析", "## 💰 Token Savings Analysis"
    ).replace(
        "## 💰 Token节省分析", "## 💰 Token Savings Analysis"
    ).replace(
        "### 单智能体场景", "### Single Agent Scenario"
    ).replace(
        "### 多智能体场景", "### Multi-Agent Scenario"
    ).replace(
        "**不使用Context Sharing**", "**Without Context Sharing**"
    ).replace(
        "**使用Context Sharing**", "**With Context Sharing**"
    ).replace(
        "**节省**", "**Savings**"
    ).replace(
        "### 总体节省", "### Total Savings"
    ).replace(
        "**总节省金额**", "**Total Token Savings**"
    ).replace(
        "**总Token节省**", "**Total Token Savings**"
    ).replace(
        "**总节省比例**", "**Total Savings Percentage**"
    ).replace(
        "**每轮节省**", "**Per Round Savings**"
    ).replace(
        "**平均每轮节省**", "**Average Per Round Savings**"
    ).replace(
        "## 🎯 使用建议", "## 🎯 Usage Recommendations"
    ).replace(
        "### 何时使用Context Sharing", "### When to Use Context Sharing"
    ).replace(
        "### 性能优化建议", "### Performance Optimization"
    ).replace(
        "### 成本优化建议", "### Cost Optimization"
    ).replace(
        "### 架构考虑", "### Architecture Considerations"
    ).replace(
        "## 📋 总结", "## 📋 Summary"
    ).replace(
        "本次测试验证了PC Node在Context Sharing方面的性能表现：",
        "This test validates the performance of PC Node's Context Sharing capabilities:"
    ).replace(
        "**单智能体场景**: Context Sharing带来了", "**Single Agent Scenario**: Context Sharing achieved"
    ).replace(
        "**多智能体场景**: Context Sharing带来了", "**Multi-Agent Scenario**: Context Sharing achieved"
    ).replace(
        "的Token效率提升", " token efficiency improvement"
    ).replace(
        "**Token节省**: 平均每轮对话节省", "**Token Savings**: Average per round savings -  "
    ).replace(
        "**规模效应**: 每1000轮对话节省", "**Scale Projection**: Savings per 1,000 rounds - "
    ).replace(
        "*报告由PC Node自动生成 | 数据来源: 综合性能测试*",
        "*Report automatically generated by PC Node | Data source: Comprehensive performance testing*"
    ).replace(
        "单智能体场景显示", "Single agent scenario shows"
    ).replace(
        "多智能体场景显示", "Multi-agent scenario shows"
    ).replace(
        "Context Sharing在多智能体环境中表现更优，适合协作型应用",
        "Context Sharing performs better in multi-agent environments, suitable for collaborative applications"
    ).replace(
        "Context Sharing有效减少Token使用，提升响应效率",
        "Context Sharing effectively reduces token usage and improves response efficiency"
    ).replace(
        "通过Context Sharing可显著降低API调用成本",
        "Context Sharing significantly reduces API call costs"
    ).replace(
        "tokens", "tokens"
    ).replace(
        "优���", "Excellent"
    ).replace(
        "良好", "Good"
    ).replace(
        "一般", "Average"
    ).replace(
        "需要优化", "Needs Optimization"
    )

    # 使用正则表达式处理复杂情况
    import re

    # 清理乱码字符
    english_content = re.sub(r'[��]+', '', english_content)

    # 处理推荐语的翻译
    english_content = re.sub(r'，推荐使用', ', recommended for use', english_content)
    english_content = re.sub(r'，强烈推荐使用', ', highly recommended', english_content)

    # 修复token efficiency improvement后面直接跟推荐语的情况
    english_content = re.sub(r'improvement推荐使用', 'improvement, recommended for use', english_content)
    english_content = re.sub(r'improvement强烈推荐使用', 'improvement, highly recommended', english_content)
    english_content = re.sub(r'improvementrecommended for use', 'improvement, recommended for use', english_content)
    english_content = re.sub(r'improvementhighly recommended', 'improvement, highly recommended', english_content)

    # 处理独立的推荐语翻译
    english_content = re.sub(r'推荐使用', 'recommended for use', english_content)
    english_content = re.sub(r'强烈推荐使用', 'highly recommended', english_content)

    # 处理中文标点符号
    english_content = english_content.replace('，', ', ')
    english_content = english_content.replace('。', '. ')
    english_content = english_content.replace('：', ': ')
    english_content = english_content.replace('；', '; ')

    # 修复格式问题
    english_content = english_content.replace("achieved", "achieved ").replace("achieved  ", "achieved ")

    # 处理tokens相关的数字格式
    english_content = re.sub(r'savings: (\d+) tokens', r'savings: \1 tokens', english_content)
    english_content = re.sub(r'rounds: (\d+) tokens', r'rounds: \1 tokens', english_content)

    # 修复Summary部分的tokens表述
    # english_content = re.sub(r'3\. \*\*Token Savings\*\*: Average per round savings: (\d+) tokens', r'3. **Token Savings**: Average \1 tokens per conversation turn', english_content)
    # english_content = re.sub(r'4\. \*\*Scale Projection\*\*: Savings per 1,000 rounds: (\d+) tokens', r'4. **Scale Projection**: \1 tokens savings per 1,000 conversation turns', english_content)

    # 最后的格式清理
    english_content = re.sub(r'[ \t]+', ' ', english_content)
    english_content = re.sub(r' +', ' ', english_content)
    english_content = re.sub(r' +\n', '\n', english_content)

    # 保存英文报告
    with open(english_report_path, 'w', encoding='utf-8') as f:
        f.write(english_content)

    return english_report_path


def main():
    """主函数"""
    print("🚀 开始生成PC Node性能报告...")
    print("="*60)

    # 1. 运行完整测试
    print("📋 步骤 1: 运行性能测试")
    runner = TestRunner()
    test_results = runner.run_all_tests()

    if not test_results.get('tests_completed'):
        print("❌ 测试失败，无法生成报告")
        return 1

    # 2. 创建docs目录结构
    print("\n📁 步骤 2: 创建文档目录结构")
    docs_dir, images_dir = create_docs_structure()

    # 3. 移动图表到docs/images
    print("\n📊 步骤 3: 移动图表文件")
    test_results_dir = runner.results_dir
    chart_files = move_charts_to_docs(test_results_dir, images_dir)

    # 4. 生成综合分析报告
    print("\n📈 步骤 4: 生成综合分析���告")
    analyzer = TestDataAnalyzer()

    # 获取测试结果
    single_results = test_results.get('single_agent_results', {})
    multi_results = test_results.get('multi_agent_results', {})

    if not single_results or not multi_results:
        print("❌ 测试结果不完整，无法生成综合分析")
        return 1

    # 生成分析
    analysis = analyzer.analyze_comprehensive_results(single_results, multi_results)

    # 生成中文报告
    chinese_report_path = os.path.join(docs_dir, "pc_node_performance_report.zh.md")
    analyzer.generate_markdown_report(analysis, chinese_report_path)

    # 5. 生成英文报告
    print("\n🌍 步骤 5: 生成英文版报告")
    english_report_path = generate_english_report(chinese_report_path, docs_dir)

    # 6. 输出总结
    print("\n" + "="*60)
    print("✅ PC Node性能报告生成完成！")
    print("="*60)

    print(f"�� 中文报告: {chinese_report_path}")
    print(f"📄 英文报告: {english_report_path}")
    print(f"📊 图表目录: {images_dir}")
    print(f"📈 生成的图表: {', '.join(chart_files)}")

    # 显示关键性能指标
    if 'comprehensive_analysis' in test_results:
        analysis = test_results['comprehensive_analysis']
        single_efficiency = analysis['test_summary']['single_agent'].get('improvements', {}).get('token_efficiency', 0)
        multi_efficiency = analysis['test_summary']['multi_agent'].get('improvements', {}).get('token_efficiency', 0)
        total_token_savings = analysis['cost_analysis']['total_savings']['tokens']

        print(f"\n🎯 关键性能指标:")
        print(f"   🤖 单智能体Token效率提升: {single_efficiency:.1f}%")
        print(f"   👥 多智能体Token效率提升: {multi_efficiency:.1f}%")
        print(f"   💰 总体Token节省: {total_token_savings:,.0f} tokens")

    print("\n��� 使用建议:")
    print("   1. 将报告文件添加到Git仓库")
    print("   2. 在README中引用性能报告")
    print("   3. 定期运行此脚本更新性能数据")

    return 0


if __name__ == "__main__":
    sys.exit(main())
