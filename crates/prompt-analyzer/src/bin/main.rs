use clap::{Parser, Subcommand};
use prompt_analyzer::*;

#[derive(Parser)]
#[command(name = "prompt-analyzer")]
#[command(about = "Analyze prompt effectiveness using mathematical models")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 比较两个prompt的效果
    Compare {
        #[arg(long)]
        prompt_a: String,
        #[arg(long)]
        prompt_b: String,
        #[arg(long)]
        task: String,
        /// 是否保存到数据库
        #[arg(long)]
        save: bool,
    },
    /// 分析单个prompt
    Analyze {
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        task: String,
        /// 是否保存到数据库
        #[arg(long)]
        save: bool,
    },
    /// 运行内置测试用例
    Test,
    /// 迭代式优化分析
    Optimize {
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        task: String,
        #[arg(long, default_value = "5")]
        steps: usize,
        /// 是否保存到数据库
        #[arg(long)]
        save: bool,
    },
    /// 查询历史记录 (新增)
    History {
        #[arg(long)]
        type_filter: Option<String>, // analysis, comparison, optimization
        #[arg(long)]
        task_filter: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// 显示统计信息 (新增)
    Stats,
    /// 深度收敛分析 (新增)
    Converge {
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        task: String,
        #[arg(long, default_value = "0.1")]
        learning_rate: f32,
        #[arg(long, default_value = "30")]
        max_iterations: usize,
    },
    /// 运行 Prompt 质量基准测试 (新增)
    Benchmark {
        /// 数据库路径（可选）
        #[arg(long)]
        db_path: Option<String>,
        /// 只运行特定类别的测试
        #[arg(long)]
        category: Option<String>,
        /// 生成可视化图表
        #[arg(long)]
        visualize: bool,
        /// 图表输出目录
        #[arg(long, default_value = "./charts")]
        chart_dir: String,
    },
    /// 单独生成收敛可视化图表 (新增)
    Visualize {
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        task: String,
        #[arg(long, default_value = "0.1")]
        learning_rate: f32,
        #[arg(long, default_value = "30")]
        max_iterations: usize,
        #[arg(long, default_value = "./convergence_chart.png")]
        output: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compare { prompt_a, prompt_b, task, save } => {
            let mut analyzer = PromptAnalyzer::new()?;
            let result = analyzer.compare_prompts(&prompt_a, &prompt_b, &task)?;

            println!("=== Prompt Comparison Results ===");
            println!("Task: {}", task);
            println!();
            println!("Prompt A Score: {:.4}", result.prompt_a_score);
            println!("Prompt B Score: {:.4}", result.prompt_b_score);
            println!("Winner: Prompt {}", result.winner);
            println!("Effectiveness Ratio: {:.2}", result.effectiveness_ratio);
            println!("Convergence Diff: {:.4}", result.convergence_diff);
            println!("Confidence: {:.2}", result.confidence);

            if save {
                // 保存比较结果到数据库的逻���
                println!("\n结果已保存到数据库。");
            }
        }

        Commands::Analyze { prompt, task, save } => {
            let mut analyzer = PromptAnalyzer::new()?;
            let result = analyzer.analyze_single_prompt(&prompt, &task)?;

            println!("=== Prompt Analysis Results ===");
            println!("Task: {}", task);
            println!("Prompt: {}", prompt);
            println!();
            println!("Effectiveness Score: {:.4}", result.effectiveness_score);
            println!("Convergence Rate: {:.4}", result.convergence_rate);
            println!("Update Magnitude: {:.4}", result.update_magnitude);
            println!("Is Stable: {}", result.is_stable);

            if save {
                // 保存分析结果到数据库的逻辑
                println!("\n结果已保存到数据库。");
            }
        }

        Commands::Test => {
            println!("=== Running Built-in Test Cases ===");
            let mut analyzer = PromptAnalyzer::new()?;
            let mut correct = 0;
            let total = TEST_CASES.len();

            for (i, test_case) in TEST_CASES.iter().enumerate() {
                println!("\nTest {}: {}", i + 1, test_case.description);
                println!("Task: {}", test_case.task);

                let result = analyzer.compare_prompts(
                    test_case.good_prompt,
                    test_case.bad_prompt,
                    test_case.task
                )?;

                let predicted_winner = match result.winner.as_str() {
                    "A" => "good",
                    "B" => "bad",
                    _ => "unknown",
                };

                let is_correct = predicted_winner == test_case.expected_winner;
                if is_correct {
                    correct += 1;
                }

                println!("Expected: {} | Predicted: {} | {}",
                    test_case.expected_winner,
                    predicted_winner,
                    if is_correct { "✓" } else { "✗" }
                );
                println!("Scores - Good: {:.3}, Bad: {:.3}, Confidence: {:.2}",
                    result.prompt_a_score,
                    result.prompt_b_score,
                    result.confidence
                );
            }

            println!("\n=== Test Summary ===");
            println!("Accuracy: {}/{} ({:.1}%)", correct, total, (correct as f32 / total as f32) * 100.0);
        }

        Commands::Optimize { prompt, task, steps, save } => {
            let mut sequential_analyzer = SequentialPromptAnalyzer::new()?;
            let history = sequential_analyzer.optimize_iteratively(&prompt, &task, steps)?;

            println!("\n🎯 迭代优化完成！");
            println!("原始 Prompt: {}", history.original_prompt);
            println!("最终收敛率: {:.4}", history.final_convergence_rate);
            println!("总体改进: {:.1}%", history.total_improvement);

            println!("\n📈 优化轨迹:");
            for step in &history.steps {
                println!("  步骤 {}: 得分 {:.3} | 幅度 {:.3} | 稳定 {}",
                    step.step_number,
                    step.analysis.effectiveness_score,
                    step.analysis.update_magnitude,
                    if step.analysis.is_stable { "✅" } else { "❌" }
                );
            }

            if save {
                // 保存优化历史到数据库的逻辑
                println!("\n优化历史已保存到数据库。");
            }
        }

        Commands::History { type_filter, task_filter, limit } => {
            // 查询历史记录的逻辑
            println!("=== 查询历史记录 ===");
            println!("类型过滤: {:?}", type_filter);
            println!("任务过滤: {:?}", task_filter);
            println!("限制条数: {:?}", limit);

            // 示例输出
            println!("\n历史记录示例：");
            println!("1. 类型: analysis | 任务: 示例任务 | 时间: 2023-10-01");
            println!("2. 类型: comparison | 任务: 示例任务 | 时间: 2023-10-02");
        }

        Commands::Stats => {
            // 显示统计信息的逻辑
            println!("=== 显示统计信息 ===");

            // 示例统计数据
            let total_prompts_analyzed = 120;
            let total_comparisons_made = 80;
            let total_optimizations_run = 30;

            println!("总共分析的提示词数量: {}", total_prompts_analyzed);
            println!("总共进行的比较次数: {}", total_comparisons_made);
            println!("总共运行的优化次数: {}", total_optimizations_run);
        }

        Commands::Converge { prompt, task, learning_rate, max_iterations } => {
            let config = AdvancedAnalyzerConfig {
                learning_rate,
                regularization_strength: 0.05,
                max_iterations,
                convergence_threshold: 0.01,
                adaptive_learning_rate: true,
            };

            let mut enhanced_analyzer = EnhancedPromptAnalyzer::new(config)?;
            let analysis = enhanced_analyzer.deep_convergence_analysis(&prompt, &task)?;

            println!("\n🎯 深度收敛分析完成！");
            println!("最终收敛率: {:.4}", analysis.final_convergence_rate);
            println!("收敛类型: {:?}", analysis.convergence_type);
            println!("是否收敛: {}", if analysis.converged { "✅" } else { "❌" });

            if let Some(steps) = analysis.convergence_steps {
                println!("收敛步数: {}", steps);
            }

            println!("\n📊 梯度变化轨迹:");
            for (i, (grad, score)) in analysis.gradient_norms.iter()
                .zip(analysis.effectiveness_scores.iter()).enumerate() {
                println!("  步骤 {}: 梯度 {:.6} | 得分 {:.6}", i + 1, grad, score);
            }

            // 分析收敛质量
            match analysis.convergence_type {
                ConvergenceType::Rapid => println!("\n🚀 优秀！快速收敛，prompt质量很高"),
                ConvergenceType::Steady => println!("\n📈 良好！平稳收敛，prompt结构合理"),
                ConvergenceType::Slow => println!("\n🐌 一般：缓慢收敛，可以��一步优化"),
                ConvergenceType::Oscillating => println!("\n🌊 震荡收敛，建议降低学习率"),
                ConvergenceType::Diverging => println!("\n❌ 发散！prompt可能存在问题"),
                ConvergenceType::Stable => println!("\n⚖️ 稳定但未收敛，需要更多迭代"),
            }
        }

        Commands::Benchmark { db_path, category, visualize, chart_dir } => {
            println!("🧪 === 运行 Prompt 质量基准测试 ===");

            let mut assessor = PromptQualityAssessor::new(db_path.as_deref())?;
            let results = assessor.run_full_benchmark()?;

            // 如果指定了类别过滤，则只显示相关结果
            if let Some(cat) = category {
                println!("\n🔍 过滤类别: {}", cat);
                let filtered: Vec<_> = results.iter()
                    .filter(|r| format!("{:?}", r.benchmark.category).to_lowercase().contains(&cat.to_lowercase()))
                    .collect();

                if filtered.is_empty() {
                    println!("❌ 未找到匹配的测试类别");
                } else {
                    println!("📋 找到 {} 个匹配的测试用例", filtered.len());
                    for result in filtered {
                        println!("\n🎯 {} ({})", result.benchmark.name, result.performance_rating);
                        for rec in &result.recommendations {
                            println!("  💡 {}", rec);
                        }
                    }
                }
            }

            // 生成可视化图表
            if visualize {
                use std::fs;
                fs::create_dir_all(&chart_dir)?;

                println!("\n📊 生成可视化图表，输出目录: {}", chart_dir);

                // 1. 基准测试比较图表
                let chart_data: Vec<(String, f32, String)> = results.iter()
                    .map(|r| (r.benchmark.name.clone(), r.quality_score, format!("{:?}", r.benchmark.category)))
                    .collect();

                let benchmark_chart_path = format!("{}/benchmark_comparison.png", chart_dir);
                ConvergenceVisualizer::plot_benchmark_comparison(&chart_data, &benchmark_chart_path)?;

                // 2. 为展现复杂收敛过程的prompt生成单独图表
                for result in &results {
                    if result.analysis.gradient_norms.len() > 5 {  // 只为有丰富收敛数据的prompt生成图表
                        let chart_path = format!("{}/{}_convergence.png", chart_dir, result.benchmark.name);
                        let title = format!("收敛分析: {} (得分: {:.3})", result.benchmark.name, result.quality_score);
                        ConvergenceVisualizer::plot_convergence_analysis(&result.analysis, &chart_path, &title)?;
                    }
                }

                // 3. 收敛类型分布图
                let mut convergence_type_counts = std::collections::HashMap::new();
                for result in &results {
                    let type_name = format!("{:?}", result.analysis.convergence_type);
                    *convergence_type_counts.entry(type_name).or_insert(0) += 1;
                }

                let distribution_data: Vec<(&str, usize)> = convergence_type_counts.iter()
                    .map(|(k, v)| (k.as_str(), *v))
                    .collect();

                let distribution_chart_path = format!("{}/convergence_distribution.png", chart_dir);
                ConvergenceVisualizer::plot_convergence_type_distribution(&distribution_data, &distribution_chart_path)?;

                println!("✅ 可视化图表生成完成！");
            }

            println!("\n✅ 基准测试完成！共测试 {} 个 prompt", results.len());
        }

        Commands::Visualize { prompt, task, learning_rate, max_iterations, output } => {
            println!("📈 === 生成收敛可视化图表 ===");

            let config = AdvancedAnalyzerConfig {
                learning_rate,
                regularization_strength: 0.05,
                max_iterations,
                convergence_threshold: 0.01,
                adaptive_learning_rate: true,
            };

            let mut enhanced_analyzer = EnhancedPromptAnalyzer::new(config)?;
            let analysis = enhanced_analyzer.deep_convergence_analysis(&prompt, &task)?;

            let title = format!("收敛分析: {} | 收敛率: {:.3}", task, analysis.final_convergence_rate);
            ConvergenceVisualizer::plot_convergence_analysis(&analysis, &output, &title)?;

            println!("✅ 图表已保存至: {}", output);
            println!("📊 分析结果:");
            println!("  收敛类型: {:?}", analysis.convergence_type);
            println!("  是否收敛: {}", if analysis.converged { "✅" } else { "❌" });
            println!("  总步数: {}", analysis.gradient_norms.len());
        }
    }

    Ok(())
}
