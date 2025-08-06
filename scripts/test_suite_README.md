# PC Node 性能测试套件

重构后的PC Node性能测试系统，提供更清晰的测试结构和更准确的性能对比。

## 📁 文件结构

```
scripts/
├── test_runner.py              # 主测试运行器
├── test_single_agent.py        # 单智能体对比测试
├── test_multi_agent.py         # 多智能体对比测试
├── test_data_analyzer.py       # 数据分析和报告生成
├── utils/                      # 工具模块
│   ├── __init__.py
│   ├── pc_client.py            # PC Node客户端封装
│   ├── performance_metrics.py  # 性能指标计算
│   └── chart_generator.py      # 图表生成工具
├── configs/                    # 配置文件
│   ├── single_agent_config.json
│   └── multi_agent_config.json
└── test_results/               # 测试结果目录（自动创建）
```

## 🧪 测试设计

### 核心对比逻辑

**单智能体测试** (`test_single_agent.py`):
- 场景A: 20轮对话，不开启Context Sharing（传递完整消息历史）
- 场景B: 20轮对话，开启Context Sharing（只传递当前消息）
- 对比指标: Token使用量、响应时间、压缩效果

**多智能体测试** (`test_multi_agent.py`):
- 场景A: 3个智能体×20轮对话，不开启Context Sharing
- 场景B: 3个智能体×20轮对话，开启Context Sharing
- 对比指标: 跨智能体知识共享、总Token消耗、协作效率

## 🚀 快速开始

### 1. 运行完整测试套件

```bash
cd scripts
python test_runner.py
```

### 2. 运行单项测试

```bash
# 只运行单智能体测试
python test_runner.py --skip-multi

# 只运行多智能体测试  
python test_runner.py --skip-single

# 快速测试模式
python test_runner.py --quick
```

### 3. 直接运行单个测试

```bash
# 单智能体对比测试
python test_single_agent.py

# 多智能体对比测试
python test_multi_agent.py
```

## 📊 输出结果

### 生成的文件

- `single_agent_YYYYMMDD_HHMMSS.json` - 单智能体测试原始数据
- `multi_agent_YYYYMMDD_HHMMSS.json` - 多智能体测试原始数据
- `complete_test_results_YYYYMMDD_HHMMSS.json` - 完整测试结果
- `analysis_report_YYYYMMDD_HHMMSS.md` - Markdown格式分析报告

### 生成的图表

- `single_agent_comparison.png` - 单智能体性能对比
- `multi_agent_overall_comparison.png` - 多智能体整体对比
- `multi_agent_without_context.png` - 多智能体分析（不开启Context Sharing）
- `multi_agent_with_context.png` - 多智能体分析（开启Context Sharing）

## ⚙️ 配置说明

### 单智能体配置 (`configs/single_agent_config.json`)

```json
{
  "test_config": {
    "base_url": "http://localhost:3000",
    "model": "gpt-3.5-turbo", 
    "temperature": 0.7,
    "max_tokens": 150,
    "conversation_rounds": 20
  },
  "conversation_scenarios": {
    "web_scraping": [
      // 20个网页抓取相关的连续问题
    ]
  }
}
```

### 多智能体配置 (`configs/multi_agent_config.json`)

```json
{
  "test_config": {
    // 基础配置同上
  },
  "agents": {
    "sales_manager_001": {
      "name": "销售经理",
      "role": "负责客户沟通和需求分析"
    },
    // 更多智能体配置
  },
  "conversation_scenarios": {
    "enterprise_project": [
      // 20轮企业项目讨论场景
    ]
  }
}
```

## 🔧 依赖要求

```bash
pip install requests matplotlib numpy
```

## 📈 关键性能指标

### Token效率
- **Token节省率**: Context Sharing相比传统方法的Token减少百分比
- **平均Token使用**: 每轮对话的平均Token消耗
- **Token增长控制**: 长对话中Token使用的稳定性

### 响应性能
- **平均响应时间**: API调用的平均耗时
- **响应时间稳定性**: 响应时间的变异系数

### 压缩效果
- **压缩比**: Context压缩的效率
- **压缩稳定性**: 压缩效果的一致性

### 多智能体协作
- **跨智能体知识传递**: 智能体间信息共享效果
- **协作效率**: 多智能体场景下的整体性能

## 🎯 测试场景说明

### 单智能体场景: 网页抓取教学
一个连续的20轮对话，模拟用户学习网页抓取技术的过程。每个问题都基于前面的内容，测试上下文保持能力。

### 多智能体场景: 企业项目讨论
3个不同角色的智能体（销售经理、技术负责人、项目经理）参与一个企业CRM项目的讨论，共20轮对话，测试跨团队知识共享。

## 🔍 故障排除

### 常见问题

1. **PC Node连接失败**
   - 确保PC Node服务在 `http://localhost:3000` 运行
   - 检查网络连接和防火墙设置

2. **导入错误**
   - 确保在 `scripts/` 目录下运行测试
   - 检查Python路径设置

3. **图表生成失败** 
   - 安装matplotlib: `pip install matplotlib`
   - 检查字体设置，某些系统可能需要额外配置

4. **内存或性能问题**
   - 使用 `--quick` 参数减少测试轮数
   - 调整配置文件中的 `conversation_rounds` 参数

## 📝 自定义测试

### 添加新的测试场景

1. 修改配置文件中的 `conversation_scenarios`
2. 调整 `conversation_rounds` 参数
3. 重新运行测试

### 修改性能指标

编辑 `utils/performance_metrics.py` 中的计算逻辑来添加新的性能指标。

### 自定义图表

修改 `utils/chart_generator.py` 来调整图表样式或添加新的可视化。

## 🤝 贡献

欢迎提交Issue和Pull Request来改进测试套件！

## 📜 许可证

遵循项目主许可证。
