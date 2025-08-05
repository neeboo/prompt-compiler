pub struct PromptTestCase {
    pub good_prompt: &'static str,
    pub bad_prompt: &'static str,
    pub task: &'static str,
    pub expected_winner: &'static str,
    pub description: &'static str,
}

pub const TEST_CASES: &[PromptTestCase] = &[
    PromptTestCase {
        good_prompt: "Please analyze following the steps: 1) Understand the problem background 2) List key points 3) Provide specific conclusions",
        bad_prompt: "analyze this",
        task: "analyze market trends",
        expected_winner: "good",
        description: "Structured vs vague instructions",
    },
    PromptTestCase {
        good_prompt: "As a professional translator, please accurately translate the following content while maintaining the original tone and technical terminology",
        bad_prompt: "translate:",
        task: "translate technical documents",
        expected_winner: "good",
        description: "Professional role vs simple instruction",
    },
    PromptTestCase {
        good_prompt: "Please answer the question in this format:\nConclusion: [your conclusion]\nReason: [supporting reasons]\nSuggestion: [specific recommendations]",
        bad_prompt: "answer the question",
        task: "answer consulting questions",
        expected_winner: "good",
        description: "Formatted output vs open response",
    },
    PromptTestCase {
        good_prompt: "Please explain the concept in detail and provide specific examples",
        bad_prompt: "explain this",
        task: "explain technical concepts",
        expected_winner: "good",
        description: "Specific requirements vs vague requirements",
    },
];
