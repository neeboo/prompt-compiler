# Prompt Compiler 🧠

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/neeboo/prompt-compiler)

[English](./README.md) | 中文文档

基于突破性研究论文《Learning without training: The implicit dynamics of in-context learning》的最先进AI提示符编译器。使用transformer理论中的隐式权重更新动力学来编译、优化和版本控制自然语言提示符。

## 🎯 核心理念

基于重要的理论发现：**上下文学习(ICL)等价于MLP层中的隐式低秩权重更新**

```
T_W(C,x) = T_{W+ΔW(C)}(x)
```

其中 `ΔW(C)` 代表由上下文C生成的**rank-1权重更新**。

## ✨ 主要功能

- 🔄 **Prompt编译**: 将自然语言提示符转换为优化的中间表示(IR)
- ⚡ **智能优化**: 基于权重更新理论的多层优化策略
- 🌳 **版本控制**: Git风格的DAG版本管理系统用于提示符演进
- 🔐 **密码学验证**: Ed25519签名确保数据完整性
- 🗄️ **高性能存储**: RocksDB持久化存储与高效索引
- 📊 **权重动力学分析**: 实时计算和分析隐式权重更新

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- Git
- 8GB+ RAM (用于大型权重矩阵计算)

### 安装

```bash
# 克隆仓库
git clone https://github.com/neeboo/prompt-compiler.git
cd prompt-compiler

# 构建项目
cargo build --release

# 可选：安装到系统路径
cargo install --path .
```

### 验证安装

```bash
# 检查版本
pc version

# 运行权重动力学演示
pc weight-demo -c 3 --verbose
```

### 基础使用

```bash
# 使用权重分析编译提示符
pc compile -p "你是一个专业的Rust开发者。帮我优化代码性能" \
  -m gpt-4 --enable-weight-analysis

# 分析提示符质量
pc analyze -p "写一个排序算法" -a semantic

# 使用约束优化提示符
pc optimize -p "帮我写代码" -O all -b 500

# 演示权重更新动力学
pc weight-demo -c 5 --verbose
```

## 🏗️ 项目结构

```
prompt-compiler/                    # Monorepo根目录
├── crates/                        # 📦 所有包
│   ├── prompt-compiler-core/      # 🧠 核心编译器逻辑
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 库入口点
│   │       ├── error.rs          # 错误处理
│   │       ├── ir.rs             # 中间表示
│   │       └── compiler/         # 编译管道
│   │           ├── mod.rs        # 主编译器逻辑
│   │           ├── analyzers/    # 语义分析，上下文学习
│   │           ├── optimizers/   # 权重更新，token预算优化器
│   │           └── generators/   # 标准和权重感知生成器
│   │
│   ├── prompt-compiler-cli/       # 🖥️ 命令行接口
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # CLI入口点 (pc命令)
│   │       ├── lib.rs            # CLI库
│   │       └── cli.rs            # 命令实现
│   │
│   ├── prompt-compiler-weights/   # ⚖️ 权重动力学计算
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # 权重更新理论实现
│   │
│   ├── prompt-compiler-storage/   # 🗄️ 持久化层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 存储模块入口
│   │       ├── state_db.rs       # RocksDB状态管理
│   │       └── dag.rs            # 版本化DAG操作
│   │
│   ├── prompt-compiler-crypto/    # 🔐 密码学工具
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # Ed25519签名和哈希
│   │
│   ├── prompt-compiler-web/       # 🌐 Web界面和API
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Web服务器入口 (pc-server命令)
│   │       └── lib.rs            # REST API实现
│   │
│   └── prompt-compiler-sdk/       # 📚 集成SDK
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs            # 应用程序高级SDK
│
├── docs/                          # 📚 文档
├── examples/                      # 🔍 使用示例
├── tests/                         # 🧪 集成测试
├── benches/                       # ⚡ 性能基准测试
├── Cargo.toml                     # 🏗️ 工作空间配置
├── config.toml                    # ⚙️ 默认配置
├── README.md                      # 📖 项目文档
└── .gitignore                     # 🚫 Git忽略规则
```

### 核心架构

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   原始提示符    │───▶│   编译器管道     │───▶│   优化提示符    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │   权重动力学     │
                    │   (理论基础)     │
                    └──────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │   持久化存储     │
                    │ (RocksDB + DAG)  │
                    └──────────────────┘
```

### 包依赖关系

```
prompt-compiler-cli ────┐
                        ├──▶ prompt-compiler-core ────┐
prompt-compiler-web ────┘                            │
                                                     ├──▶ prompt-compiler-weights
prompt-compiler-sdk ──────▶ prompt-compiler-storage ─┤
                                                     └──▶ prompt-compiler-crypto
```

## 🔧 技术栈

### 核心后端
- **Rust** - 系统编程语言，确保性能和安全性
- **RocksDB** - 高性能键值存储，用于状态持久化
- **Ed25519** - 椭圆曲线数字签名算法

### 数学计算
- **nalgebra** - 线性代数库，用于权重矩阵计算
- **ndarray** - N维数组操作

### CLI和工具
- **clap** - 命令行参数解析
- **serde** - 序列化/反序列化框架
- **tokio** - 异步运行时

### 编译管道

```
原始提示符 → 分析器 → 优化器 → 权重更新计算 → 生成器 → 优化提示符
     ↓           ↓         ↓           ↓           ↓
   语义       结构      动力学       模型        最终
   分析   → 优化    → 计算     → 适配    →    输出
```

## 📦 依赖说明

### 核心依赖
```toml
# 数值计算
nalgebra = "0.32"         # 线性代数运算
ndarray = "0.15"          # 多维数组

# 存储
rocksdb = "0.21"          # 高性能数据库

# 密码学
ed25519-dalek = "2.0"     # 数字签名
sha2 = "0.10"             # 哈希函数

# 序列化
serde = "1.0"             # 序列化框架
serde_json = "1.0"        # JSON支持

# CLI
clap = "4.0"              # 命令行解析
colored = "2.0"           # 彩色输出

# 异步
tokio = "1.0"             # 异步运行时
```

## 🔬 理论实现

### 权重更新计算
基于研究论文的核心算法：

```rust
// 计算rank-1权重更新
pub fn compute_weight_update(
    pretrained_weights: &Matrix,
    context_vector: &Vector,
    query_vector: &Vector,
) -> Matrix {
    let w_delta_a = pretrained_weights * context_vector;
    let query_norm_sq = query_vector.norm_squared();
    
    // ΔW = (W·ΔA) · A^T / ||A||²
    (w_delta_a * query_vector.transpose()) / query_norm_sq
}
```

## 🧠 理论基础

### 隐式权重更新

基于研究发现，我们的编译器实现了：

1. **上下文向量化**: 将提示符上下文转换为向量表示
2. **权重更新计算**: 使用公式 `ΔW = (W·ΔA) · A^T / ||A||²`
3. **收敛性分析**: 预测权重更新的收敛行为
4. **优化策略**: 基于权重更新效果调整提示符结构

### 关键公式

```
// 权重更新公式
ΔW(Y) = (W·ΔA(Y)) · A(C\Y,x)^T / ||A(C\Y,x)||²

// 序列化学习动力学
W_i = W_{i-1} - h·∇_W L_i(W_{i-1})

// 其中 h = 1/||A(x)||² 是自适应学习率
```

## 📊 使用示例

### 1. 基础编译

```bash
pc compile -p "请写一个时间复杂度为O(n log n)的高效排序算法" --enable-weight-analysis
```

输出：
```
🚀 开始提示符编译...
📊 权重更新分析已启用
📝 编译结果:
==================================================
## 任务目标
请写一个时间复杂度为O(n log n)的高效排序算法

## 执行指令
- 请将回答控制在1000个token以内
- 任务优先级: 5/10
- 已应用权重更新优化
==================================================

📊 编译统计:
目标模型: gpt-4
Token预算: 1000
优先级: 5/10
权重更新数量: 3
效果评分: 0.745
已收敛: 是
```

### 2. 权重动力学演示

```bash
pc weight-demo -c 5 --verbose
```

输出：
```
🧠 权重更新动力学演示
基于论文: Learning without training: The implicit dynamics of in-context learning

🔢 计算序列化权重更新...

📊 权重更新序列:
步骤 1: 更新范数 = 0.2345, 效果 = 0.1234
   上下文向量范数: 0.8765
   查询向量范数: 0.9876
步骤 2: 更新范数 = 0.1987, 效果 = 0.2145
   ...

🎯 收敛分析:
收敛率: 0.8234
已收敛: 是

💡 这演示了论文中描述的隐式权重更新机制
```

## 🔧 配置

创建 `config.toml`:

```toml
[compiler]
default_model = "gpt-4"
default_token_budget = 1000
enable_weight_analysis = true
analyzers = ["semantic", "context"]
optimizers = ["weight", "budget"]

[storage]
database_path = "./prompt_compiler_db"
enable_compression = true
max_cache_size = 1000

[crypto]
enable_signing = true
key_path = "~/.prompt_compiler/keys"
```

## 🧪 测试和基准

```bash
# 运行测试
cargo test

# 性能基准测试
cargo bench

# 集成测试
cargo test --test integration
```

## 📋 开发路线图

### 阶段1: 核心实现 ✅
- [x] 基础编译器架构
- [x] 权重更新动力学计算
- [x] RocksDB存储层
- [x] CLI接口

### 阶段2: 增强功能 🔄
- [ ] 多模型适配器
- [ ] 分布式权重计算
- [ ] Web界面
- [ ] API服务

### 阶段3: 生态扩展 📅
- [ ] 插件系统
- [ ] 云端同步
- [ ] 机器学习模型集成
- [ ] 实时协作功能

## 🤝 贡献指南

### 开发工作流
1. **Fork** 此仓库
2. **创建分支**: `git checkout -b feature/your-feature`
3. **开发**: 遵循Rust最佳实践
4. **测试**: `cargo test && cargo bench`
5. **提交**: `git commit -m "feat: add amazing feature"`
6. **推送**: `git push origin feature/your-feature`
7. **PR**: 创建Pull Request

### 代码标准
- 使用 `cargo fmt` 进行代码格式化
- 使用 `cargo clippy` 进行代码质量检查
- 为新功能添加测试
- 更新相关文档

## 📊 性能基准

在M1 MacBook Pro上的测试结果：

| 操作         | 平均时间 | 内存使用 |
| ------------ | -------- | -------- |
| 基础编译     | ~50ms    | 12MB     |
| 权重更新计算 | ~200ms   | 45MB     |
| 存储操作     | ~5ms     | 8MB      |
| DAG遍历      | ~10ms    | 15MB     |

*基准测试基于1000次操作的平均值*

## 🔍 使用案例

### 1. 学术研究
- 验证论文中的理论假设
- 分析不同提示符结构的收敛性
- 比较权重更新效果

### 2. 工程应用
- 优化生产环境中的AI提示符
- 版本控制提示符演进
- 自动化提示符质量评估

### 3. 教育培训
- 可视化上下文学习过程
- 理解transformer内部机制
- 实践AI系统优化

## 📚 相关研究

- **主要理论基础**: [Learning without training: The implicit dynamics of in-context learning](https://arxiv.org/abs/2507.16003)
- **Transformer架构**: [Attention is All You Need](https://arxiv.org/abs/1706.03762)
- **上下文学习**: [Language Models are Few-Shot Learners](https://arxiv.org/abs/2005.14165)

## 📄 许可证

本项目采用MIT许可证 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

特别感谢基础研究的作者们：
- Benoit Dherin, Michael Munn, Hanna Mazzawi, Michael Wunder, Javier Gonzalvo (Google Research)

---

**构建更智能的AI交互** 🚀
