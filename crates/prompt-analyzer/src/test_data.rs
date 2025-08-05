pub struct PromptTestCase {
    pub good_prompt: &'static str,
    pub bad_prompt: &'static str,
    pub task: &'static str,
    pub expected_winner: &'static str,
    pub description: &'static str,
}

pub const TEST_CASES: &[PromptTestCase] = &[
    PromptTestCase {
        good_prompt: "请按照以下步骤详细分析：1) 理解问题背景 2) 列出关键要点 3) 给出具体结论",
        bad_prompt: "分析一下",
        task: "分析市场趋势",
        expected_winner: "good",
        description: "结构化 vs 模糊指令",
    },
    PromptTestCase {
        good_prompt: "作为专业翻译，请准确翻译以下内容，保持原文的语调和专业术语",
        bad_prompt: "翻译：",
        task: "翻译技术文档",
        expected_winner: "good",
        description: "专业角色 vs 简单指令",
    },
    PromptTestCase {
        good_prompt: "请按此格式回答问题：\n结论：[你的结论]\n理由：[支持理由]\n建议：[具体建议]",
        bad_prompt: "回答问题",
        task: "回答咨询问题",
        expected_winner: "good",
        description: "格式化输出 vs 开放回答",
    },
    PromptTestCase {
        good_prompt: "请详细解释概念，并给出具体例子说明",
        bad_prompt: "解释一下这个",
        task: "解释技术概念",
        expected_winner: "good",
        description: "具体要求 vs 模糊要求",
    },
];
