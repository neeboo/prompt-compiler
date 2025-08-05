// PC与LLM交互边界详解
// 展示Context在PC和LLM之间的分布和传递机制

use std::collections::HashMap;

/// 🎯 Context分层架构
///
/// ```
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                    PC (Complete Context)                        │
/// │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐│
/// │  │ Full User   │ │ Full Agent  │ │ Cross-Agent │ │ Historical  ││
/// │  │ Profile     │ │ History     │ │ Knowledge   │ │ Patterns    ││
/// │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘│
/// └─────────────────────────────────────────────────────────────────┘
///                                │
///                                │ Context Compression & Filtering
///                                ▼
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                LLM (Minimal Context)                            │
/// │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                │
/// │  │ Key User    │ │ Relevant    │ │ Task-Specific│               │
/// │  │ Info        │ │ History     │ │ Knowledge    │               │
/// │  └─────────────┘ └─────────────┘ └─────────────┘                │
/// └─────────────────────────────────────────────────────────────────┘
/// ```

/// 完整Context（PC内部）
#[derive(Debug, Clone)]
pub struct CompleteContext {
    pub user_profile: CompleteUserProfile,
    pub agent_history: CompleteAgentHistory,
    pub cross_agent_knowledge: CrossAgentKnowledge,
    pub domain_expertise: DomainExpertise,
    pub relationship_graph: RelationshipGraph,
    pub metadata: ContextMetadata,
}

/// LLM Context（发送给LLM的压缩版本）
#[derive(Debug, Clone)]
pub struct LLMContext {
    pub essential_user_info: String,    // 压缩的用户关键信息
    pub relevant_history: String,       // 相关历史摘要
    pub task_knowledge: String,         // 任务相关知识
    pub personalization_hints: String,  // 个性化提示
}

/// 完整用户档案（PC内部）
#[derive(Debug, Clone)]
pub struct CompleteUserProfile {
    pub user_id: String,
    pub demographics: HashMap<String, String>,
    pub interaction_history: Vec<InteractionRecord>,
    pub preference_analysis: PreferenceAnalysis,
    pub satisfaction_trends: Vec<SatisfactionPoint>,
    pub behavioral_patterns: BehavioralPatterns,
    pub relationship_network: Vec<String>, // 关联的其他用户
    pub privacy_settings: PrivacySettings,
}

/// 完整Agent历史（PC内部）
#[derive(Debug, Clone)]
pub struct CompleteAgentHistory {
    pub agent_id: String,
    pub all_conversations: Vec<ConversationFull>,
    pub performance_metrics: PerformanceHistory,
    pub learning_trajectory: LearningTrajectory,
    pub collaboration_patterns: CollaborationPatterns,
    pub specialization_areas: Vec<SpecializationArea>,
}

/// Context压缩引擎
pub struct ContextCompressionEngine {
    compression_rules: CompressionRules,
    relevance_scorer: RelevanceScorer,
    privacy_filter: PrivacyFilter,
    token_budget_manager: TokenBudgetManager,
}

impl ContextCompressionEngine {
    /// 🔥 核心方法：将完整Context压缩为LLM Context
    pub fn compress_context_for_llm(
        &self,
        complete_context: &CompleteContext,
        current_query: &str,
        llm_constraints: &LLMConstraints,
    ) -> Result<LLMContext, Box<dyn std::error::Error>> {

        println!("🔄 Compressing complete context for LLM...");

        // Step 1: 分析当前查询的需求
        let query_analysis = self.analyze_query_requirements(current_query);
        println!("   📋 Query type: {} | Complexity: {:.1}",
                query_analysis.query_type, query_analysis.complexity);

        // Step 2: 计算相关性分数
        let relevance_scores = self.calculate_relevance_scores(
            complete_context,
            &query_analysis
        );

        // Step 3: 应用隐私过滤
        let filtered_context = self.apply_privacy_filter(
            complete_context,
            &relevance_scores
        );

        // Step 4: Token预算分配
        let budget_allocation = self.allocate_token_budget(
            &filtered_context,
            llm_constraints.max_context_tokens
        );

        // Step 5: 生成压缩的用户信息
        let essential_user_info = self.compress_user_profile(
            &complete_context.user_profile,
            &query_analysis,
            budget_allocation.user_info_tokens
        );

        // Step 6: 生成相关历史摘要
        let relevant_history = self.compress_history(
            &complete_context.agent_history,
            &query_analysis,
            budget_allocation.history_tokens
        );

        // Step 7: 提取任务相关知识
        let task_knowledge = self.extract_task_knowledge(
            &complete_context.cross_agent_knowledge,
            &complete_context.domain_expertise,
            &query_analysis,
            budget_allocation.knowledge_tokens
        );

        // Step 8: 生成个性化提示
        let personalization_hints = self.generate_personalization_hints(
            &complete_context.user_profile,
            &query_analysis,
            budget_allocation.personalization_tokens
        );

        let llm_context = LLMContext {
            essential_user_info,
            relevant_history,
            task_knowledge,
            personalization_hints,
        };

        // 验证压缩结果
        let estimated_tokens = self.estimate_llm_context_tokens(&llm_context);
        println!("   ✅ Context compressed: {} → {} tokens ({:.1}% reduction)",
                self.estimate_complete_context_tokens(complete_context),
                estimated_tokens,
                (1.0 - estimated_tokens as f64 / self.estimate_complete_context_tokens(complete_context) as f64) * 100.0
        );

        Ok(llm_context)
    }

    /// 压缩用户档案
    fn compress_user_profile(
        &self,
        profile: &CompleteUserProfile,
        query_analysis: &QueryAnalysis,
        token_budget: u32,
    ) -> String {
        let mut compressed = String::new();

        // 根据查询类型选择关键信息
        match query_analysis.query_type.as_str() {
            "technical_support" => {
                compressed.push_str(&format!("User: {} ({}), technical level: {}",
                    profile.user_id,
                    profile.demographics.get("education").unwrap_or(&"unknown".to_string()),
                    self.assess_technical_level(profile)
                ));
            },
            "customer_service" => {
                compressed.push_str(&format!("Customer: {} ({}), communication style: {}, satisfaction: {:.1}",
                    profile.user_id,
                    profile.demographics.get("age").unwrap_or(&"unknown".to_string()),
                    self.determine_communication_style(profile),
                    self.calculate_average_satisfaction(profile)
                ));
            },
            "sales" => {
                compressed.push_str(&format!("Prospect: {} ({}), budget level: {}, decision style: {}",
                    profile.user_id,
                    profile.demographics.get("income").unwrap_or(&"unknown".to_string()),
                    self.assess_budget_level(profile),
                    self.determine_decision_style(profile)
                ));
            },
            _ => {
                compressed.push_str(&format!("User: {}", profile.user_id));
            }
        }

        // 添加最重要的偏好信息
        if let Some(key_preferences) = self.extract_key_preferences(profile, query_analysis) {
            compressed.push_str(&format!(". Preferences: {}", key_preferences));
        }

        // 确保不超过token预算
        self.truncate_to_budget(&compressed, token_budget)
    }

    /// 压缩历史记录
    fn compress_history(
        &self,
        history: &CompleteAgentHistory,
        query_analysis: &QueryAnalysis,
        token_budget: u32,
    ) -> String {
        // 找到最相关的历史对话
        let relevant_conversations = self.find_relevant_conversations(
            &history.all_conversations,
            query_analysis
        );

        let mut compressed = String::new();

        if !relevant_conversations.is_empty() {
            compressed.push_str("Recent relevant interactions: ");

            for (i, conv) in relevant_conversations.iter().take(3).enumerate() {
                if i > 0 { compressed.push_str("; "); }
                compressed.push_str(&format!("{}. {} → {}",
                    i + 1,
                    self.summarize_user_message(&conv.user_message),
                    self.summarize_agent_response(&conv.agent_response)
                ));
            }
        }

        self.truncate_to_budget(&compressed, token_budget)
    }

    /// 提取任务相关知识
    fn extract_task_knowledge(
        &self,
        cross_agent_knowledge: &CrossAgentKnowledge,
        domain_expertise: &DomainExpertise,
        query_analysis: &QueryAnalysis,
        token_budget: u32,
    ) -> String {
        let mut knowledge = String::new();

        // 获取相关的跨Agent经验
        if let Some(relevant_experience) = self.find_relevant_cross_agent_experience(
            cross_agent_knowledge,
            query_analysis
        ) {
            knowledge.push_str(&format!("Similar cases: {}", relevant_experience));
        }

        // 获取领域专业知识
        if let Some(domain_info) = self.find_relevant_domain_knowledge(
            domain_expertise,
            query_analysis
        ) {
            if !knowledge.is_empty() { knowledge.push_str(". "); }
            knowledge.push_str(&format!("Domain context: {}", domain_info));
        }

        self.truncate_to_budget(&knowledge, token_budget)
    }

    /// 生成个性化提示
    fn generate_personalization_hints(
        &self,
        profile: &CompleteUserProfile,
        query_analysis: &QueryAnalysis,
        token_budget: u32,
    ) -> String {
        let mut hints = String::new();

        // 基于用户行为模式的提示
        let communication_style = self.determine_communication_style(profile);
        hints.push_str(&format!("Adjust tone for {} communication", communication_style));

        // 基于历史满意度的提示
        let satisfaction_level = self.calculate_average_satisfaction(profile);
        if satisfaction_level < 0.7 {
            hints.push_str(". Extra care needed - previous dissatisfaction");
        }

        self.truncate_to_budget(&hints, token_budget)
    }

    /// 构建发送给LLM的最终Prompt
    pub fn build_llm_prompt(
        &self,
        llm_context: &LLMContext,
        agent_type: &str,
        user_query: &str,
    ) -> String {
        format!(
            "You are a {} agent.\n\n\
            Context: {}\n\n\
            History: {}\n\n\
            Knowledge: {}\n\n\
            Instructions: {}\n\n\
            User Query: {}\n\n\
            Please provide a helpful response:",
            agent_type,
            llm_context.essential_user_info,
            llm_context.relevant_history,
            llm_context.task_knowledge,
            llm_context.personalization_hints,
            user_query
        )
    }
}

/// 🎯 实际示例：展示Context分布
pub struct ContextDistributionDemo;

impl ContextDistributionDemo {
    pub fn demonstrate_context_flow(&self) {
        println!("🔄 Context Distribution Demo");
        println!("{}", "=".repeat(50));

        // 1. PC内部的完整Context
        self.show_complete_context();

        // 2. 压缩过程
        self.show_compression_process();

        // 3. 发送给LLM的Context
        self.show_llm_context();

        // 4. LLM响应处理
        self.show_response_processing();
    }

    fn show_complete_context(&self) {
        println!("\n📚 PC内部完整Context (约2000 tokens):");
        println!("   👤 Complete User Profile:");
        println!("      - 基本信息: 张先生, 35岁, 本科, 工程师, 中等收入");
        println!("      - 交互历史: 23次对话, 平均满意度8.5/10");
        println!("      - 行为模式: 偏好技术细节, 简洁沟通, 周二上午最活跃");
        println!("      - 关联网络: 与李工程师有协作关系");
        println!("      - 隐私设置: 允许基本信息共享, 敏感信息限制");

        println!("   🤖 Complete Agent History:");
        println!("      - 所有对话: 156次交互记录");
        println!("      - 性能指标: 解决率94%, 首次解决率78%");
        println!("      - 学习轨迹: 在登录问题上从60%提升到95%成功率");
        println!("      - 协作模式: 与技术支持Agent协作度0.8");

        println!("   🔄 Cross-Agent Knowledge:");
        println!("      - 相似案例: 技术支持Agent解决的3个类似登录问题");
        println!("      - 最佳实践: 登录问题的标准处理流程");
        println!("      - 失败案例: 2个未成功解决的复杂案例");
    }

    fn show_compression_process(&self) {
        println!("\n🔄 Context压缩过程:");
        println!("   📊 相关性分析:");
        println!("      - 用户基本信息: 相关性 0.9 (高)");
        println!("      - 最近3次交互: 相关性 0.8 (高)");
        println!("      - 技术背景: 相关性 0.9 (高)");
        println!("      - 关联网络: 相关性 0.3 (低) → 过滤掉");

        println!("   🔒 隐私过滤:");
        println!("      - 保留: 技术水平, 沟通偏好");
        println!("      - 过滤: 具体收入数字, 详细个人信息");

        println!("   💰 Token预算分配 (总预算: 300 tokens):");
        println!("      - 用户信息: 80 tokens");
        println!("      - 相关历史: 120 tokens");
        println!("      - 任务知识: 80 tokens");
        println!("      - 个性化提示: 20 tokens");
    }

    fn show_llm_context(&self) {
        println!("\n📤 发送给LLM的压缩Context (300 tokens):");
        println!("   👤 Essential User Info:");
        println!("      \"User: 张先生 (本科), technical level: high, communication style: concise, satisfaction: 8.5\"");

        println!("   📚 Relevant History:");
        println!("      \"Recent relevant interactions: 1. login issue → cache clear resolved; 2. password reset → guided successfully\"");

        println!("   🧠 Task Knowledge:");
        println!("      \"Similar cases: 3 successful login resolutions via cache clearing. Domain context: login issues typically browser-related\"");

        println!("   🎯 Personalization Hints:");
        println!("      \"Adjust tone for concise communication\"");
    }

    fn show_response_processing(&self) {
        println!("\n🔄 LLM响应处理:");
        println!("   📥 LLM原始响应 (150 tokens):");
        println!("      \"Based on your technical background and previous successful resolution, I recommend clearing your browser cache...\"");

        println!("   🔄 PC后处理:");
        println!("      - 质量评估: 8.7/10");
        println!("      - 个性化增强: 添加技术细节链接");
        println!("      - 上下文更新: 记录本次成功解决方案");
        println!("      - 跨Agent共享: 添加到登录问题知识库");

        println!("   📤 返回给Agent的最终响应:");
        println!("      \"根据您的技术背景和之前的成功经验，建议清除浏览器缓存。具体步骤：...\"");
    }
}

/// 🔧 支持结构定义
#[derive(Debug)]
struct QueryAnalysis {
    query_type: String,
    complexity: f64,
    required_context_types: Vec<String>,
    urgency: f64,
}

#[derive(Debug)]
struct LLMConstraints {
    max_context_tokens: u32,
    max_total_tokens: u32,
    response_time_limit: u32,
}

#[derive(Debug)]
struct TokenBudgetAllocation {
    user_info_tokens: u32,
    history_tokens: u32,
    knowledge_tokens: u32,
    personalization_tokens: u32,
}

// 简化的支持结构
#[derive(Debug, Clone)]
struct InteractionRecord { user_message: String, agent_response: String, timestamp: u64 }
#[derive(Debug, Clone)]
struct PreferenceAnalysis { communication_style: String, technical_level: f64 }
#[derive(Debug, Clone)]
struct SatisfactionPoint { score: f64, timestamp: u64 }
#[derive(Debug, Clone)]
struct BehavioralPatterns { active_hours: Vec<u8>, response_time_preference: u32 }
#[derive(Debug, Clone)]
struct PrivacySettings { allow_basic_sharing: bool, sensitive_data_restricted: bool }
#[derive(Debug, Clone)]
struct ConversationFull { user_message: String, agent_response: String, quality_score: f64 }
#[derive(Debug, Clone)]
struct PerformanceHistory { resolution_rate: f64, first_contact_resolution: f64 }
#[derive(Debug, Clone)]
struct LearningTrajectory { skill_improvements: HashMap<String, f64> }
#[derive(Debug, Clone)]
struct CollaborationPatterns { partner_agents: HashMap<String, f64> }
#[derive(Debug, Clone)]
struct SpecializationArea { domain: String, proficiency: f64 }
#[derive(Debug, Clone)]
struct CrossAgentKnowledge { shared_experiences: Vec<String> }
#[derive(Debug, Clone)]
struct DomainExpertise { knowledge_base: HashMap<String, String> }
#[derive(Debug, Clone)]
struct RelationshipGraph { connections: HashMap<String, f64> }
#[derive(Debug, Clone)]
struct ContextMetadata { last_updated: u64, version: String }

// 简化的引擎组件
struct CompressionRules;
struct RelevanceScorer;
struct PrivacyFilter;
struct TokenBudgetManager;

// 简化的实现
impl ContextCompressionEngine {
    fn analyze_query_requirements(&self, _query: &str) -> QueryAnalysis {
        QueryAnalysis {
            query_type: "technical_support".to_string(),
            complexity: 0.7,
            required_context_types: vec!["user_profile".to_string(), "history".to_string()],
            urgency: 0.8,
        }
    }

    fn calculate_relevance_scores(&self, _context: &CompleteContext, _analysis: &QueryAnalysis) -> HashMap<String, f64> {
        [("user_profile".to_string(), 0.9), ("history".to_string(), 0.8)].iter().cloned().collect()
    }

    fn apply_privacy_filter(&self, context: &CompleteContext, _scores: &HashMap<String, f64>) -> CompleteContext {
        context.clone()
    }

    fn allocate_token_budget(&self, _context: &CompleteContext, max_tokens: u32) -> TokenBudgetAllocation {
        TokenBudgetAllocation {
            user_info_tokens: max_tokens / 4,
            history_tokens: max_tokens / 2,
            knowledge_tokens: max_tokens / 4,
            personalization_tokens: max_tokens / 10,
        }
    }

    fn estimate_complete_context_tokens(&self, _context: &CompleteContext) -> u32 { 2000 }
    fn estimate_llm_context_tokens(&self, _context: &LLMContext) -> u32 { 300 }
    fn assess_technical_level(&self, _profile: &CompleteUserProfile) -> String { "high".to_string() }
    fn determine_communication_style(&self, _profile: &CompleteUserProfile) -> String { "concise".to_string() }
    fn calculate_average_satisfaction(&self, _profile: &CompleteUserProfile) -> f64 { 8.5 }
    fn assess_budget_level(&self, _profile: &CompleteUserProfile) -> String { "medium".to_string() }
    fn determine_decision_style(&self, _profile: &CompleteUserProfile) -> String { "analytical".to_string() }
    fn extract_key_preferences(&self, _profile: &CompleteUserProfile, _analysis: &QueryAnalysis) -> Option<String> {
        Some("technical details, quick resolution".to_string())
    }
    fn truncate_to_budget(&self, text: &str, _budget: u32) -> String { text.to_string() }
    fn find_relevant_conversations(&self, _conversations: &[ConversationFull], _analysis: &QueryAnalysis) -> Vec<ConversationFull> {
        vec![]
    }
    fn summarize_user_message(&self, message: &str) -> String { message.chars().take(20).collect::<String>() + "..." }
    fn summarize_agent_response(&self, response: &str) -> String { response.chars().take(20).collect::<String>() + "..." }
    fn find_relevant_cross_agent_experience(&self, _knowledge: &CrossAgentKnowledge, _analysis: &QueryAnalysis) -> Option<String> {
        Some("3 successful login resolutions".to_string())
    }
    fn find_relevant_domain_knowledge(&self, _expertise: &DomainExpertise, _analysis: &QueryAnalysis) -> Option<String> {
        Some("login issues typically browser-related".to_string())
    }
}

fn main() {
    println!("🔄 PC-LLM Context Boundary Demonstration");
    println!("{}", "=".repeat(60));

    let demo = ContextDistributionDemo;
    demo.demonstrate_context_flow();

    println!("\n🎯 Key Insights:");
    println!("   💾 PC maintains complete context (2000+ tokens)");
    println!("   📤 LLM receives compressed context (300 tokens)");
    println!("   🔄 86% context compression without losing effectiveness");
    println!("   🔒 Privacy-aware filtering protects sensitive data");
    println!("   🎯 Task-specific optimization for relevance");

    println!("\n✨ This demonstrates how PC acts as an intelligent context gateway!");
}
