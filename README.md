# Prompt Compiler 🧠

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/neeboo/prompt-compiler)

English | [中文文档](./README.zh.md)

A state-of-the-art AI prompt compiler based on the groundbreaking research paper: [**"Learning without training: The implicit dynamics of in-context learning"**](https://arxiv.org/html/2507.16003v1). This tool compiles, optimizes, and version-controls natural language prompts using implicit weight update dynamics from transformer theory.

## 📄 Research Foundation

This project implements the theoretical framework from:

**Citation**: *Learning without training: The implicit dynamics of in-context learning* (2024). arXiv preprint arXiv:2507.16003. Available at: https://arxiv.org/html/2507.16003v1

## 🎯 Core Concept

Built upon the fundamental discovery that **in-context learning (ICL) is equivalent to implicit low-rank weight updates in MLP layers**:

```
T_W(C,x) = T_{W+ΔW(C)}(x)
```

Where `ΔW(C)` represents a **rank-1 weight update** generated from context C.

## ✨ Key Features

- 🔄 **Prompt Compilation**: Transform natural language prompts into optimized intermediate representations (IR)
- ⚡ **Weight Dynamics Analysis**: Real-time computation of implicit weight updates based on ICL theory
- 🚀 **PC Node Context Sharing**: Advanced context sharing system with 90%+ token efficiency improvement
- 🤖 **Multi-Agent Support**: Comprehensive multi-agent conversation system with context optimization
- 📊 **Automated Performance Testing**: Complete testing framework for single-agent and multi-agent scenarios
- 📈 **Performance Analytics**: Automated report generation with detailed token usage and cost analysis
- 🌳 **Version Control**: Git-style DAG version management system for prompt evolution
- 🔐 **Cryptographic Verification**: Ed25519 signatures ensure data integrity
- 🗄️ **High-Performance Storage**: RocksDB persistent storage with efficient indexing
- 📊 **Semantic Compression**: Advanced context compression with 70%+ efficiency

## 🚀 Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/neeboo/prompt-compiler.git
cd prompt-compiler
cargo build --release
```

### Basic Usage

```bash
# 1. Configure environment variables (Required)
cp .env.example .env
# Edit .env file and add your OpenAI API Key

# 2. Build the project
cargo build --release

# 3. Run PC Node Context Sharing performance tests
cd scripts
python test_runner.py  # Run comprehensive performance tests

# 4. Generate automated performance reports
python generate_performance_report.py  # Generate complete performance analysis

# 5. View generated reports
# English: docs/pc_node_performance_report.md
# Chinese: docs/pc_node_performance_report.zh.md
```

**💡 Environment Configuration**:
- Copy `.env.example` to `.env`
- Configure your OpenAI API Key in `.env`
- Get API Key: https://platform.openai.com/api-keys

## 🏆 PC Node Context Sharing Performance

Our advanced Context Sharing system delivers exceptional performance improvements:

### **Single Agent Scenario**
- **Token Efficiency Improvement**: 90.3%
- **Token Savings**: 28,727 tokens per test cycle
- **Per Round Savings**: 1,512 tokens per conversation turn

### **Multi-Agent Scenario**  
- **Token Efficiency Improvement**: 91.3%
- **Token Savings**: 36,103 tokens per test cycle
- **Per Round Savings**: 1,805 tokens per conversation turn

### **Overall Impact**
- **Total Token Savings**: 64,830 tokens across test scenarios
- **Average Efficiency**: 90.9% token reduction
- **Scale Projection**: 1,659,000 tokens saved per 1,000 conversation turns
- **Multi-Agent Advantage**: Context Sharing performs even better in collaborative environments

📊 **Detailed Performance Report**: [PC Node Performance Analysis](./docs/pc_node_performance_report.md)

## 📊 Benchmarks & Performance

### Weight Dynamics Engine Performance
Our implementation of the ICL weight update theory delivers:

- **Convergence Rate**: ~10⁴ precision in 50-100 iterations
- **Memory Efficiency**: 70%+ compression ratio for semantic content
- **Throughput**: 1000+ prompts/second on modern hardware
- **Storage**: RocksDB with microsecond-level query performance

### Real-world Examples Performance

| Demo | Description | Performance Metrics |
|------|-------------|-------------------|
| `complete_rocksdb_demo` | Enterprise-grade semantic system | 30% compression, 0.1ms queries |
| `weight_dynamics_system` | ICL theory implementation | 10⁻⁴ convergence in <100 iterations |
| `semantic_compression_demo` | Context compression technology | 70%+ compression with semantic integrity |
| `industry_embedding_demo` | Production embedding generation | 1000+ embeddings/sec with caching |
| `pc_node_context_sharing_demo` | Advanced context sharing system | 90%+ token efficiency improvement |

## 🏗️ Architecture

```
prompt-compiler/
├── 📦 crates/                     # Core library collection
│   ├── 🧠 prompt-compiler-core/   # Compilation engine & IR
│   ├── 🔗 prompt-compiler-node/   # PC Node context sharing system
│   ├── ⚖️  prompt-compiler-weights/ # ICL weight dynamics
│   ├── 🗄️  prompt-compiler-storage/ # RocksDB persistence
│   ├── 🔐 prompt-compiler-crypto/  # Ed25519 verification
│   ├── 🌐 prompt-compiler-web/     # REST API server
│   └── 📚 prompt-compiler-sdk/     # Integration SDK
├── 🔍 examples/                   # Usage demos & benchmarks
├── 📊 benches/                    # Performance tests
├── 🧪 scripts/                    # Automated testing & reporting
│   ├── test_runner.py             # Comprehensive test execution
│   ├── test_data_analyzer.py      # Performance data analysis
│   ├── generate_performance_report.py # Automated report generation
│   └── utils/                     # Testing utilities
└── 📖 docs/                       # Documentation & reports
    ├── pc_node_performance_report.md    # English performance report
    ├── pc_node_performance_report.zh.md # Chinese performance report
    └── images/                    # Performance charts & visualizations
```

## 🔍 Examples & Demos

Explore our comprehensive example suite in [`examples/`](./examples/):

- **`complete_rocksdb_demo`**: Full enterprise semantic system
- **`weight_dynamics_system`**: ICL weight update implementation
- **`semantic_compression_demo`**: Context compression technology
- **`web_api_semantic_server`**: Production-ready API server
- **`industry_embedding_demo`**: High-performance embedding generation
- **`pc_node_context_sharing_demo`**: Advanced context sharing system

See [`examples/README.md`](./examples/README.md) for detailed usage instructions.

## 📚 Documentation

- 📖 [API Documentation](https://docs.rs/prompt-compiler)
- 🔧 [Configuration Guide](./docs/configuration.md)
- 🚀 [Quick Start Examples](./examples/)
- 📊 [Performance Benchmarks](./benches/)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.

## 🗓️ Roadmap & TODO

### Priority 2: Theoretical Validation Enhancements
- [ ] **Hyperparameter Sensitivity Analysis**
  - Systematic testing of learning rate variations (0.001 - 1.0)
  - Regularization strength impact analysis (0.0 - 0.1)
  - Multi-head attention configuration optimization
  - Convergence behavior under different parameter combinations
  - Statistical significance testing for parameter effects

### Priority 3: Engineering Optimizations
- [ ] **Batch Processing Support**
  - Vectorized weight update operations
  - Parallel context processing for multiple queries
  - Memory-efficient batch handling for large datasets
  - Streaming API for continuous processing

- [ ] **GPU Acceleration**
  - CUDA/OpenCL kernels for matrix operations
  - GPU-accelerated softmax and attention computation
  - Memory transfer optimization between CPU/GPU
  - Benchmark comparisons: CPU vs GPU performance

- [ ] **Memory Optimization**
  - Smart caching strategies for frequently accessed contexts
  - Memory pool management for large embeddings
  - Lazy loading for historical weight updates
  - Memory profiling and leak detection tools
  - Zero-copy operations where possible

### Future Enhancements
- [ ] **Multi-Agent Context Sharing System**
- [ ] **Distributed Computing Support**
- [ ] **Real-time Monitoring Dashboard**
- [ ] **Integration with Popular ML Frameworks**

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

Built upon the theoretical foundation of implicit in-context learning dynamics. Special thanks to the research community advancing our understanding of transformer mechanisms.
