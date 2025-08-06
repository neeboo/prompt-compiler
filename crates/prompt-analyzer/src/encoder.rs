use nalgebra::DVector;

pub struct SimpleTextEncoder;

impl SimpleTextEncoder {
    pub fn new() -> Self {
        Self
    }

    /// 将prompt文本编码为特征向量
    pub fn encode_prompt(&self, text: &str) -> DVector<f32> {
        let text_lower = text.to_lowercase();

        let features = vec![
            // 长度特征 (归一化)
            (text.len() as f32 / 100.0).min(3.0),

            // 结构化指标
            text.matches("步骤").count() as f32,
            text.matches("按照").count() as f32,
            text.matches("首先").count() as f32,
            text.matches("然后").count() as f32,

            // 礼貌程度
            text.matches("请").count() as f32,
            text.matches("麻烦").count() as f32,

            // 具体化程度  
            text.matches("详细").count() as f32,
            text.matches("具体").count() as f32,
            text.matches("清楚").count() as f32,

            // 专业性指标
            text.matches("作为").count() as f32,
            text.matches("专业").count() as f32,
            text.matches("根据").count() as f32,

            // 示例和格式
            text.matches("例如").count() as f32,
            text.matches("格式").count() as f32,
            text.matches("按此").count() as f32,
        ];

        DVector::from_vec(features)
    }

    /// 将任务描述编码为查询向量
    pub fn encode_task(&self, task: &str) -> DVector<f32> {
        let task_lower = task.to_lowercase();

        let features = vec![
            // 任务类型特征
            task_lower.contains("分析") as i32 as f32,
            task_lower.contains("翻译") as i32 as f32,
            task_lower.contains("总结") as i32 as f32,
            task_lower.contains("创作") as i32 as f32,
            task_lower.contains("回答") as i32 as f32,
            task_lower.contains("解释") as i32 as f32,
            task_lower.contains("比较") as i32 as f32,
            task_lower.contains("评估") as i32 as f32,
        ];

        DVector::from_vec(features)
    }

    /// 获取特征维度
    pub fn prompt_feature_dim(&self) -> usize {
        16 // 对应encode_prompt中的特征数量
    }

    pub fn task_feature_dim(&self) -> usize {
        8 // 对应encode_task中的特征数量  
    }
}
