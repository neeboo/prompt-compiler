# Prompt Compiler 🧠

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/neeboo/prompt-compiler)

[English](./README.md) | 中文文档

基于突破性研究论文 [**《Learning without training: The implicit dynamics of in-context learning》**](https://arxiv.org/html/2507.16003v1) 的最先进AI提示符编译器。使用transformer理论中的隐式权重更新动力学来编译、优化和版本控制自然语言提示符。

## 📄 研究基础

本项目实现了以下论文的理论框架：

**引用**: *Learning without training: The implicit dynamics of in-context learning* (2024). arXiv preprint arXiv:2507.16003. 可访问：https://arxiv.org/html/2507.16003v1

## 🎯 核心理念

基于重要的理论发现：**上下文学习(ICL)等价于MLP层中的隐式低秩权重更新**

```
T_W(C,x) = T_{W+ΔW(C)}(x)
```

其中 `ΔW(C)` 代表由上下文C生成的**rank-1权重更新**。

## ✨ 主要功能

- 🔄 **提示符编译**: 将自然语言提示符转换为优化的中间表示(IR)
- ⚡ **权重动力学分析**: 基于ICL理论的实时隐式权重更新计算
- 🚀 **PC Node上下文共享**: 先进的上下文共享系统，Token效率提升90%+
- 🤖 **多智能体支持**: 全面的多智能体对话系统与上下文优化
- 📊 **自动化性能测试**: 完整的单智能体和多智能体场景测试框架
- 📈 **性能分析**: 自动化报告生成，包含详细的Token使用量和成本分析
- 🌳 **版本控制**: Git风格的DAG版本管理系统用于提示符演进
- 🔐 **密码学验证**: Ed25519签名确保数据完整性
- 🗄️ **高性能存储**: RocksDB持久化存储与高效索引
- 📊 **语义压缩**: 高级上下文压缩，效率超过70%

## 🚀 快速开始

### 安装

```bash
# 克隆并构建
git clone https://github.com/neeboo/prompt-compiler.git
cd prompt-compiler
cargo build --release
```

### 基本使用

```bash
# 1. 配置环境变量 (必须)
cp .env.example .env
# 编辑 .env 文件，添加你的 OpenAI API Key

# 2. 构建项目
cargo build --release

# 3. 运行PC Node性能测试
cd scripts
python test_runner.py  # 运行综合性能测试

# 4. 生成自动化性能报告
python generate_performance_report.py  # 生成完整的性能分析报告

# 5. 查看生成的报告
# 中文版: docs/pc_node_performance_report.zh.md
# 英文版: docs/pc_node_performance_report.md
```

**💡 环境配置说明**:
- 复制 `.env.example` 为 `.env`
- 在 `.env` 中配置你的 OpenAI API Key
- API Key 获取: https://platform.openai.com/api-keys

## 🏆 PC Node上下文共享性能

我们的先进上下文共享系统带来了卓越的性能提升：

### **单智能体场景**
- **Token效率提升**: 90.3%
- **Token节省**: 每测试周期节省28,727个tokens
- **每轮节省**: 每次对话轮次节省1,512个tokens

### **多智能体场景**  
- **Token效率提升**: 91.3%
- **Token节省**: 每测试周期节省36,103个tokens
- **每轮节省**: 每次对话轮次节省1,805个tokens

### **整体影响**
- **总Token节省**: 跨测试场景总计节省64,830个tokens
- **平均效率**: 90.9%的token减少率
- **规模预测**: 每1000轮对话可节省1,659,000个tokens
- **多智能体优势**: 在协作环境中上下文共享表现更佳

📊 **详细性能报告**: [PC Node性能分析报告](./docs/pc_node_performance_report.zh.md)

## 📊 基准测试与性能

### 权重动力学引擎性能
我们的ICL权重更新理论实现提供：

- **收敛速度**: 50-100次迭代达到~10⁻⁴精��
- **内存效率**: 语义内容压缩比超过70%
- **吞吐量**: 现代硬件上每秒1000+提示符处理
- **存储性能**: RocksDB微秒级查询响应

### 实际示例性能表现

| 演示程序 | 描述 | 性能指标 |
|---------|------|---------|
| `complete_rocksdb_demo` | 企业级语义系统 | 30%压缩率，0.1ms查询 |
| `weight_dynamics_system` | ICL理论实现 | <100次迭代达到10⁻⁴收敛 |
| `semantic_compression_demo` | 上下文压缩技术 | 70%+压缩率保持语义完整性 |
| `industry_embedding_demo` | 生产级embedding生成 | 缓存优化1000+embedding/秒 |

## 🏗️ 架构

```
prompt-compiler/
├── 📦 crates/                     # 核心库集合
│   ├── 🧠 prompt-compiler-core/   # 编译引擎与IR
│   ├── ⚖️  prompt-compiler-weights/ # ICL权重动力学
│   ├── 🗄️  prompt-compiler-storage/ # RocksDB持久化
│   ├── 🔐 prompt-compiler-crypto/  # Ed25519验证
│   ├── 🌐 prompt-compiler-web/     # REST API服务器
│   └── ���� prompt-compiler-sdk/     # 集成SDK
├── 🔍 examples/                   # 使用演示与基准测试
└── 📊 benches/                    # 性能测试
```

## 🔍 示例与演示

探索 [`examples/`](./examples/) 中的综合示例套件：

- **`complete_rocksdb_demo`**: ��整企业语义系统
- **`weight_dynamics_system`**: ICL权重更新实现
- **`semantic_compression_demo`**: 上下文压缩技术
- **`web_api_semantic_server`**: 生产就绪API服务器
- **`industry_embedding_demo`**: 高性能embedding生成

详细使用说明请参见 [`examples/README.md`](./examples/README.md)。

## 📚 文档

- 📖 [API文档](https://docs.rs/prompt-compiler)
- 🔧 [配置指南](./docs/configuration.md)
- 🚀 [快速开始示例](./examples/)
- 📊 [性能基准测试](./benches/)

## 🤝 贡献

我们欢迎贡献！请查看我们的[贡献指南](./CONTRIBUTING.md)了解详情。

## 🗓️ 路线图与待办事项

### 优先级2: 理论验证增强
- [ ] **超参数敏感性分析**
  - 学习率变化的系统性测试 (0.001 - 1.0)
  - 正则化强度影响分析 (0.0 - 0.1)
  - 多头注意力配置优化
  - 不同参数组合下的收敛行为
  - 参数效应的统计显著性测试

### 优先级3: 工程优化
- [ ] **批处理支持**
  - 矢量化权重更新操作
  - 多查询的并行上下文处���
  - 大数据集的内存高效批处理
  - 连续处理的流式API

- [ ] **GPU加速**
  - 矩阵操作的CUDA/OpenCL内核
  - GPU加速的softmax和注意力计算
  - CPU/GPU间内存传输优化
  - ���准对比：CPU vs GPU性能

- [ ] **内存优化**
  - 频繁访问上下文的智能缓存策略
  - 大型embedding的内存池管理
  - 历史权重更新的懒加载
  - 内存分析和泄漏检测工具
  - 零拷贝���作优化

### 未来增强功能
- [ ] **多Agent上下文共享系统**
- [ ] **分布式计算支持**
- [ ] **实时监控仪表板**
- [ ] **与主流ML框架集成**

## 📄 许可证

本项目采用MIT许可证 - 详情请见[LICENSE](./LICENSE)文件。

## 🙏 致谢

基于隐式上下文学习动力学的理论基础构建。特别感谢推进我们对transformer机制理解的���究社区。
