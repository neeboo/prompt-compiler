pub mod encoder;
pub mod analyzer;
pub mod test_data;
pub mod sequential;
pub mod storage;
pub mod enhanced;  // 新增增强版分析器
pub mod benchmark;  // 新增基准测试模块
pub mod visualizer;  // 新增可视化模块

pub use encoder::SimpleTextEncoder;
pub use analyzer::{PromptAnalyzer, PromptComparison, PromptAnalysis};
pub use test_data::{PromptTestCase, TEST_CASES};
pub use sequential::{SequentialPromptAnalyzer, OptimizationHistory, OptimizationStep};
pub use storage::{PromptAnalysisStorage, AnalysisRecord, ComparisonRecord, OptimizationRecord};
pub use enhanced::{EnhancedPromptAnalyzer, AdvancedAnalyzerConfig, DetailedConvergenceAnalysis, ConvergenceType};  // 新增
pub use benchmark::{PromptQualityAssessor, PromptBenchmark, QualityLevel, PromptCategory, BenchmarkResult};  // 新增
pub use visualizer::ConvergenceVisualizer;  // 新增
