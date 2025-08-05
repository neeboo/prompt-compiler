# Examples - Demos and Test Cases

This directory contains all demonstration programs and test cases for the enterprise-level semantic system.

## 🚀 Core System Demonstrations

### High-Performance Storage System
- **`complete_rocksdb_demo.rs`** - Complete RocksDB semantic system demonstration
- **`high_performance_rocksdb_system.rs`** - High-performance optimized version showcasing enterprise-level performance
- **`rocksdb_semantic_system.rs`** - Basic RocksDB integration demonstration
- **`persistent_semantic_system.rs`** - Persistent semantic system

### Weight Dynamics System
- **`weight_dynamics_system.rs`** - Weight update dynamics core implementation based on paper theory

### Web API Services
- **`web_api_semantic_server.rs`** - Enterprise-level Web API server integrating all core functionalities

## 🧠 AI and Machine Learning Demonstrations

### Semantic Processing
- **`semantic_compression_demo.rs`** - Semantic compression technology demonstration
- **`theory_verification.rs`** - Paper theory verification
- **`agent_memory_demo.rs`** - AI agent memory system demonstration

### Embeddings and Vectors
- **`industry_embedding_demo.rs`** - Industrial-grade embedding generation
- **`real_embedding_demo.rs`** - Real embedding API calls
- **`openai_env_demo.rs`** - OpenAI API integration demonstration

### Model Comparison
- **`model_comparison_demo.rs`** - Performance comparison of different models

## 📊 Performance and Caching

### Caching System
- **`cache_demo.rs`** - Intelligent caching mechanism demonstration
- **`simple_demo.rs`** - Simple semantic system introduction
- **`improved_demo.rs`** - Improved version demonstration

### Integrated Systems
- **`integrated_semantic_system.rs`** - Complete integrated system demonstration
- **`simple_persistent_system.rs`** - Simple persistent system

## 🗄️ Demo Configuration and Environment

### Demo-specific Configuration
- **`demo_Cargo.toml`** - Cargo configuration specific to demonstration programs
- **`demo_config.toml`** - Demo system configuration file

## 🎯 Quick Start

### Running the Examples

1. **Basic System Test**
   ```bash
   cd examples
   ./simple_demo
   ```

2. **Complete RocksDB System**
   ```bash
   cd examples
   ./complete_rocksdb_demo
   ```

3. **High-Performance System**
   ```bash
   cd examples
   ./high_performance_rocksdb_system
   ```

### Environment Setup

Create a `.env` file in the examples directory:
```env
OPENAI_API_KEY=your_openai_api_key_here
```

## 📈 System Effectiveness Testing

Run the system effectiveness test:
```bash
./test_system_effectiveness.sh
```

This will generate a comprehensive report in `system_effectiveness_report.md`.

## 🔧 Building Examples

Most examples are pre-compiled. To rebuild:
```bash
cargo build --release --examples
```

## 📚 Additional Documentation

- See `README.cn.md` for Chinese documentation
- Check individual example files for detailed comments
- Refer to `system_effectiveness_report.md` for performance analysis
