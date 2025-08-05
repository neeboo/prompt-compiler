use crate::enhanced::DetailedConvergenceAnalysis;
use plotters::prelude::*;

/// 可视化收敛分析结果
pub struct ConvergenceVisualizer;

impl ConvergenceVisualizer {
    /// 创建收敛曲线图表
    pub fn plot_convergence_analysis(
        analysis: &DetailedConvergenceAnalysis,
        output_path: &str,
        title: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_steps = analysis.gradient_norms.len();
        let max_gradient: f32 = analysis.gradient_norms.iter().fold(0.0, |a, &b| a.max(b));
        let max_effectiveness: f32 = analysis.effectiveness_scores.iter().fold(0.0, |a, &b| a.max(b));

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(80)
            .build_cartesian_2d(0..max_steps, 0.0..max_gradient.max(max_effectiveness))?;

        chart
            .configure_mesh()
            .x_desc("收敛步数")
            .y_desc("数值")
            .draw()?;

        // 绘制梯度范数曲线（蓝色）
        chart
            .draw_series(LineSeries::new(
                analysis.gradient_norms.iter().enumerate().map(|(i, &norm)| (i, norm)),
                &BLUE,
            ))?
            .label("梯度范数")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

        // 绘制效果得分曲线（红色）
        chart
            .draw_series(LineSeries::new(
                analysis.effectiveness_scores.iter().enumerate().map(|(i, &score)| (i, score)),
                &RED,
            ))?
            .label("效果得分")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));

        // 标记收敛点（如果存在）
        if let Some(convergence_step) = analysis.convergence_steps {
            chart.draw_series(std::iter::once(Circle::new(
                (convergence_step - 1, analysis.gradient_norms[convergence_step - 1]),
                5,
                GREEN.filled(),
            )))?
            .label("收敛点")
            .legend(|(x, y)| Circle::new((x + 5, y), 3, GREEN.filled()));
        }

        chart.configure_series_labels().draw()?;
        root.present()?;

        println!("📊 收敛图表已保存到: {}", output_path);
        Ok(())
    }

    /// 创建基准测试比较图表
    pub fn plot_benchmark_comparison(
        results: &[(String, f32, String)], // (name, score, category)
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_score = results.iter().map(|(_, score, _)| *score).fold(0.0, f32::max);
        let num_results = results.len();

        let mut chart = ChartBuilder::on(&root)
            .caption("Prompt 质量基准测试结果", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(150)
            .y_label_area_size(80)
            .build_cartesian_2d(0..num_results, 0.0..max_score)?;

        chart
            .configure_mesh()
            .x_desc("测试用例")
            .y_desc("质量得分")
            .x_label_formatter(&|x| {
                if *x < results.len() {
                    results[*x].0.clone()
                } else {
                    String::new()
                }
            })
            .draw()?;

        // 绘制柱状图
        chart.draw_series(
            results.iter().enumerate().map(|(i, (_, score, category))| {
                let color = match category.as_str() {
                    "Simple" => &BLUE,
                    "Structured" => &GREEN,
                    "Professional" => &RED,
                    "Complex" => &MAGENTA,
                    "Creative" => &CYAN,
                    "Analytical" => &BLACK,
                    _ => &BLUE,
                };
                Rectangle::new([(i, 0.0), (i, *score)], color.filled())
            })
        )?;

        // 添加质量等级参考线
        chart.draw_series(std::iter::once(PathElement::new(
            vec![(0, 0.75), (num_results, 0.75)],
            GREEN.stroke_width(2),
        )))?
        .label("Excellent 阈值 (0.75)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], GREEN));

        chart.draw_series(std::iter::once(PathElement::new(
            vec![(0, 0.55), (num_results, 0.55)],
            BLUE.stroke_width(2),
        )))?
        .label("Good 阈值 (0.55)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], BLUE));

        chart.configure_series_labels().draw()?;
        root.present()?;

        println!("📊 基准测试比较图表已保存到: {}", output_path);
        Ok(())
    }

    /// 创建收敛类型分布饼图
    pub fn plot_convergence_type_distribution(
        convergence_types: &[(&str, usize)], // (type_name, count)
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let total: usize = convergence_types.iter().map(|(_, count)| *count).sum();
        let colors = [&RED, &BLUE, &GREEN, &MAGENTA, &CYAN, &BLACK];

        let mut chart = ChartBuilder::on(&root)
            .caption("收敛类型分布", ("sans-serif", 40))
            .margin(10)
            .build_cartesian_2d(-1.2f32..1.2f32, -1.2f32..1.2f32)?;

        let mut start_angle = 0.0;
        for (i, (type_name, count)) in convergence_types.iter().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * (*count as f32) / (total as f32);

            // 绘制扇形
            let color = colors[i % colors.len()];
            let center = (0.0, 0.0);
            let radius = 1.0;

            // 简化的扇形绘制（使用多边形近似）
            let mut points = vec![center];
            for j in 0..=20 {
                let current_angle = start_angle + angle * (j as f32) / 20.0;
                let x = center.0 + radius * current_angle.cos();
                let y = center.1 + radius * current_angle.sin();
                points.push((x, y));
            }

            chart.draw_series(std::iter::once(Polygon::new(points, color.filled())))?
                .label(format!("{}: {} ({}%)", type_name, count, (100 * count) / total))
                .legend(move |(x, y)| Rectangle::new([(x, y), (x + 10, y + 10)], color.filled()));

            start_angle += angle;
        }

        chart.configure_series_labels().draw()?;
        root.present()?;

        println!("📊 收敛类型分布图已保存到: {}", output_path);
        Ok(())
    }
}
