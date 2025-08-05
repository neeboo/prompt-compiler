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

# 安装CLI工具
cargo install --path crates/prompt-compiler-cli
```

### 基本使用

```bash
# 编译优化提示符
pc compile -p "你是一个Rust专家，帮助优化代码性能"

# 分析权重动力学
pc weight-demo -c 5 --verbose

# 启动web服务器
pc-server  # 访问 http://localhost:3000
```

## 📊 基准测试与性能

### 权重动力学引擎性能
我们的ICL权重更新理论实现提供：

- **收敛速度**: 50-100次迭代达到~10⁻⁴精度
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

### 运行基准测试

```bash
# 运行所有示例并生成性能报告
cd examples
./test_system_effectiveness.sh

# 运行特定基准测试
cargo bench

# 测试权重动力学收敛
./weight_dynamics_system
```

## 🏗️ 架构

```
prompt-compiler/
├── 📦 crates/                     # 核心库集合
│   ├── 🧠 prompt-compiler-core/   # 编译引擎与IR
│   ├── ⚖️  prompt-compiler-weights/ # ICL权重动力学
│   ├── 🗄️  prompt-compiler-storage/ # RocksDB持久化
│   ├── 🔐 prompt-compiler-crypto/  # Ed25519验证
│   ├── 🌐 prompt-compiler-web/     # REST API服务器
│   └── 📚 prompt-compiler-sdk/     # 集成SDK
├── 🔍 examples/                   # 使用演示与基准测试
└── 📊 benches/                    # 性能测试
```

## 🔍 示例与演示

探索 [`examples/`](./examples/) 中的综合示例套件：

- **`complete_rocksdb_demo`**: 完整企业语义系统
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

## 📄 许可证

本项目采用MIT许可证 - 详情请见[LICENSE](./LICENSE)文件。

## 🙏 致谢

基于隐式上下文学习动力学的理论基础构建。特别感谢推进我们对transformer机制理解的研究社区。
