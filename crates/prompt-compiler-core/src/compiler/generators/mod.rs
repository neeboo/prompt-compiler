//! Standard prompt generator

use crate::compiler::{PromptGenerator, PromptIR, ModelTarget};
use crate::error::Result;

/// 标准 Prompt 生成器
pub struct StandardGenerator;

impl StandardGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl PromptGenerator for StandardGenerator {
    fn generate(&self, ir: &PromptIR, target: &ModelTarget) -> Result<String> {
        let mut output = ir.compiled_content.clone();

        // 根据目标模型调整输出
        match target.model_name.as_str() {
            "gpt-4" | "gpt-3.5-turbo" => {
                output = self.optimize_for_gpt(&output, target)?;
            }
            "claude" => {
                output = self.optimize_for_claude(&output)?;
            }
            _ => {
                output = self.add_generic_enhancements(&output)?;
            }
        }
        
        Ok(output)
    }
}

impl StandardGenerator {
    /// 为 GPT 模型优化
    fn optimize_for_gpt(&self, content: &str, target: &ModelTarget) -> Result<String> {
        let mut optimized = content.to_string();

        // 添加 token 预算提示
        if target.max_tokens < 500 {
            optimized.push_str("\n\n请保持回答简洁，控制在较短篇幅内。");
        }
        
        // 根据温度调整创造性提示
        if target.temperature > 0.8 {
            optimized.push_str("\n\n请发挥创造性，提供创新的解决方案。");
        } else if target.temperature < 0.3 {
            optimized.push_str("\n\n请提供准确、一致的回答。");
        }

        Ok(optimized)
    }

    /// 为 Claude 模型优化
    fn optimize_for_claude(&self, content: &str) -> Result<String> {
        let optimized = format!("Human: {}\n\nAssistant: 我理解您的要求。", content);
        Ok(optimized)
    }

    /// 通用增强
    fn add_generic_enhancements(&self, content: &str) -> Result<String> {
        let enhanced = format!("{}\n\n请确保回答准确、有用且易于理解。", content);
        Ok(enhanced)
    }
}
