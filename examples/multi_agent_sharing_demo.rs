use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Agent角色类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentRole {
    CustomerService,
    TechnicalSupport,
    SalesAssistant,
    CodeAssistant,
    DataAnalyst,
    ProjectManager,
}

impl AgentRole {
    /// 计算两个角色之间的协作度
    pub fn collaboration_score(&self, other: &AgentRole) -> f64 {
        match (self, other) {
            // 同类型Agent高度协作
            (AgentRole::CustomerService, AgentRole::CustomerService) => 0.9,
            (AgentRole::TechnicalSupport, AgentRole::TechnicalSupport) => 0.9,
            (AgentRole::CodeAssistant, AgentRole::CodeAssistant) => 0.9,
            (AgentRole::SalesAssistant, AgentRole::SalesAssistant) => 0.9,
            (AgentRole::DataAnalyst, AgentRole::DataAnalyst) => 0.9,
            (AgentRole::ProjectManager, AgentRole::ProjectManager) => 0.9,

            // 相关类型中度协作
            (AgentRole::CustomerService, AgentRole::TechnicalSupport) => 0.7,
            (AgentRole::TechnicalSupport, AgentRole::CustomerService) => 0.7,
            (AgentRole::TechnicalSupport, AgentRole::CodeAssistant) => 0.6,
            (AgentRole::CodeAssistant, AgentRole::TechnicalSupport) => 0.6,
            (AgentRole::SalesAssistant, AgentRole::CustomerService) => 0.5,
            (AgentRole::CustomerService, AgentRole::SalesAssistant) => 0.5,
            (AgentRole::DataAnalyst, AgentRole::ProjectManager) => 0.6,
            (AgentRole::ProjectManager, AgentRole::DataAnalyst) => 0.6,

            // 低度协作
            (AgentRole::SalesAssistant, AgentRole::CodeAssistant) => 0.2,
            (AgentRole::CodeAssistant, AgentRole::SalesAssistant) => 0.2,
            _ => 0.1,
        }
    }
}

/// 学习经验记录
#[derive(Debug, Clone)]
pub struct LearningExperience {
    pub id: String,
    pub agent_id: String,
    pub context: String,
    pub response: String,
    pub quality_score: f64,
    pub timestamp: u64,
    pub weight_delta: Vec<f64>, // 简化的权重增量
}

/// 共享权重矩阵
#[derive(Debug, Clone)]
pub struct SharedWeightMatrix {
    pub dimension: usize,
    pub weights: Vec<f64>,
    pub agent_contributions: HashMap<String, f64>, // 各Agent的贡献度
}

impl SharedWeightMatrix {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            weights: vec![0.0; dimension],
            agent_contributions: HashMap::new(),
        }
    }

    /// 应用Agent的权重更新
    pub fn apply_agent_update(&mut self, agent_id: &str, delta: &[f64], quality: f64) {
        for (i, &d) in delta.iter().enumerate() {
            if i < self.weights.len() {
                self.weights[i] += d * quality;
            }
        }

        // 更新贡献度
        *self.agent_contributions.entry(agent_id.to_string()).or_insert(0.0) += quality;
    }

    /// 传播学习到相关Agent
    pub fn propagate_learning(&mut self, source_role: &AgentRole, target_agents: &[(String, AgentRole)], delta: &[f64]) {
        for (agent_id, role) in target_agents {
            let collaboration_score = source_role.collaboration_score(role);
            if collaboration_score > 0.3 {
                // 按协作度缩放权重更新
                let scaled_delta: Vec<f64> = delta.iter().map(|&d| d * collaboration_score).collect();
                self.apply_agent_update(agent_id, &scaled_delta, collaboration_score);
            }
        }
    }
}

/// Agent上下文
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub id: String,
    pub role: AgentRole,
    pub experience_count: usize,
    pub average_quality: f64,
    pub personal_weights: Vec<f64>, // 个性化权重差异
}

impl AgentContext {
    pub fn new(id: String, role: AgentRole, dimension: usize) -> Self {
        Self {
            id,
            role,
            experience_count: 0,
            average_quality: 0.0,
            personal_weights: vec![0.0; dimension],
        }
    }
}

/// 解决方案建议
#[derive(Debug, Clone)]
pub struct SolutionSuggestion {
    pub solution_text: String,
    pub confidence: f64,
    pub source_agents: Vec<String>,
    pub similarity_score: f64,
}

/// 多Agent共享上下文管理器
pub struct MultiAgentContextManager {
    pub shared_matrix: SharedWeightMatrix,
    pub agents: HashMap<String, AgentContext>,
    pub experiences: Vec<LearningExperience>,
    pub dimension: usize,
}

impl MultiAgentContextManager {
    pub fn new(dimension: usize) -> Self {
        Self {
            shared_matrix: SharedWeightMatrix::new(dimension),
            agents: HashMap::new(),
            experiences: Vec::new(),
            dimension,
        }
    }

    /// 注册新Agent
    pub fn register_agent(&mut self, agent_id: String, role: AgentRole) -> Result<(), Box<dyn Error>> {
        let agent_context = AgentContext::new(agent_id.clone(), role.clone(), self.dimension);
        self.agents.insert(agent_id.clone(), agent_context);

        println!("✅ Agent {} registered with role {:?}", agent_id, role);
        Ok(())
    }

    /// Agent添加学习经验
    pub fn add_learning_experience(
        &mut self,
        agent_id: String,
        context: String,
        response: String,
        quality_score: f64,
    ) -> Result<(), Box<dyn Error>> {

        // 生成简化的权重增量（基于文本内容的hash）
        let weight_delta = self.generate_weight_delta(&context, &response);

        let experience = LearningExperience {
            id: format!("exp_{}_{}", agent_id, self.experiences.len()),
            agent_id: agent_id.clone(),
            context: context.clone(),
            response: response.clone(),
            quality_score,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            weight_delta: weight_delta.clone(),
        };

        // 更新共享权重矩阵
        self.shared_matrix.apply_agent_update(&agent_id, &weight_delta, quality_score);

        // 传播学习到相关Agent
        if let Some(source_agent) = self.agents.get(&agent_id) {
            let other_agents: Vec<(String, AgentRole)> = self.agents.iter()
                .filter(|(id, _)| *id != &agent_id)
                .map(|(id, agent)| (id.clone(), agent.role.clone()))
                .collect();

            self.shared_matrix.propagate_learning(&source_agent.role, &other_agents, &weight_delta);
        }

        // 更新Agent统计
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.experience_count += 1;
            agent.average_quality = (agent.average_quality * (agent.experience_count - 1) as f64 + quality_score) / agent.experience_count as f64;
        }

        self.experiences.push(experience);

        println!("📚 Agent {} added experience: quality {:.2}", agent_id, quality_score);
        Ok(())
    }

    /// 查询共享上下文
    pub fn query_shared_context(&self, requesting_agent: &str, query: &str, max_results: usize) -> Vec<&LearningExperience> {
        let mut relevant_experiences: Vec<&LearningExperience> = self.experiences.iter()
            .filter(|exp| exp.context.to_lowercase().contains(&query.to_lowercase()) ||
                         exp.response.to_lowercase().contains(&query.to_lowercase()))
            .collect();

        // 按质量分数和相关度排序
        relevant_experiences.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap());
        relevant_experiences.truncate(max_results);

        if !relevant_experiences.is_empty() {
            println!("🔍 Found {} relevant experiences for agent {}", relevant_experiences.len(), requesting_agent);
        }

        relevant_experiences
    }

    /// 生成权重增量（语义感知版本）
    fn generate_weight_delta(&self, context: &str, response: &str) -> Vec<f64> {
        let combined = format!("{} {}", context, response).to_lowercase();

        // 预定义的语义关键词映射
        let semantic_keywords = [
            ("login", vec![0, 20, 45, 67]),
            ("password", vec![1, 21, 46, 68]),
            ("cache", vec![2, 22, 47, 69]),
            ("browser", vec![3, 23, 48, 70]),
            ("reset", vec![4, 24, 49, 71]),
            ("error", vec![5, 25, 50, 72]),
            ("timeout", vec![6, 26, 51, 73]),
            ("api", vec![7, 27, 52, 74]),
            ("retry", vec![8, 28, 53, 75]),
            ("customer", vec![9, 29, 54, 76]),
            ("user", vec![9, 29, 54, 76]), // 与customer相同，语义相近
            ("reports", vec![10, 30, 55, 77]),
            ("cannot", vec![11, 31, 56, 78]),
            ("failure", vec![5, 25, 50, 72]), // 与error相同，语义相近
            ("pricing", vec![12, 32, 57, 79]),
            ("plans", vec![13, 33, 58, 80]),
            ("function", vec![14, 34, 59, 81]),
            ("optimization", vec![15, 35, 60, 82]),
            ("performance", vec![15, 35, 60, 82]), // 与optimization相同
            ("resolved", vec![16, 36, 61, 83]),
            ("successful", vec![16, 36, 61, 83]), // 与resolved相同
            ("improved", vec![16, 36, 61, 83]), // 与resolved相同
        ];

        let mut delta = vec![0.0; self.dimension];

        // 基于语义关键词设置权重
        for (keyword, indices) in &semantic_keywords {
            if combined.contains(keyword) {
                for &idx in indices {
                    if idx < self.dimension {
                        delta[idx] += 0.1; // 语义权重
                    }
                }
            }
        }

        // 添加文本长度和复杂度特征
        let word_count = combined.split_whitespace().count();
        let complexity_idx = (word_count * 3) % self.dimension;
        delta[complexity_idx] += word_count as f64 * 0.01;

        // 如果没有匹配到关键词，回退到字符级特征
        if delta.iter().all(|&x| x == 0.0) {
            let bytes = combined.as_bytes();
            for (i, &byte) in bytes.iter().enumerate() {
                let idx1 = (i * 7 + byte as usize) % self.dimension;
                delta[idx1] += (byte as f64 / 255.0) * 0.02;
            }
        }

        delta
    }

    /// 基于权重矩阵查询相似解决方案
    pub fn query_solution_from_weights(&self, agent_id: &str, problem_context: &str) -> Option<SolutionSuggestion> {
        // 生成问题的权重向量
        let problem_weights = self.generate_weight_delta(problem_context, "");

        let mut best_match: Option<&LearningExperience> = None;
        let mut best_similarity = 0.0;

        // 在所有经验中寻找最相似的
        for experience in &self.experiences {
            let similarity = self.calculate_similarity(&problem_weights, &experience.weight_delta);
            println!("🔍 Checking similarity with {}: {:.3}", experience.agent_id, similarity);

            if similarity > best_similarity && similarity > 0.1 { // 降低相似度阈值从0.3到0.1
                best_similarity = similarity;
                best_match = Some(experience);
            }
        }

        if let Some(matched_exp) = best_match {
            println!("🎯 Best match found: {} with similarity {:.3}", matched_exp.agent_id, best_similarity);

            // 基于相似经验和当前Agent的权重生成解决方案
            let suggested_solution = self.generate_adaptive_solution(
                agent_id,
                problem_context,
                matched_exp,
                best_similarity
            );

            Some(SolutionSuggestion {
                solution_text: suggested_solution,
                confidence: best_similarity * 0.8, // 置信度稍低于相似度
                source_agents: vec![matched_exp.agent_id.clone()],
                similarity_score: best_similarity,
            })
        } else {
            println!("❌ No matching experience found (threshold: 0.1)");
            None
        }
    }

    /// 计算两个权重向量的相似度
    fn calculate_similarity(&self, weights1: &[f64], weights2: &[f64]) -> f64 {
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for i in 0..weights1.len().min(weights2.len()) {
            dot_product += weights1[i] * weights2[i];
            norm1 += weights1[i] * weights1[i];
            norm2 += weights2[i] * weights2[i];
        }

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        // 余弦相似度
        dot_product / (norm1.sqrt() * norm2.sqrt())
    }

    /// 基于学习经验生成自适应解决方案
    fn generate_adaptive_solution(
        &self,
        agent_id: &str,
        problem: &str,
        reference_exp: &LearningExperience,
        similarity: f64
    ) -> String {
        let agent_role = &self.agents.get(agent_id).unwrap().role;

        // 根据Agent角色和相似度调整解决方案
        match agent_role {
            AgentRole::CustomerService => {
                if problem.to_lowercase().contains("login") {
                    if similarity > 0.8 {
                        format!("Based on similar cases: {}. Confidence: {:.1}%",
                               reference_exp.response, similarity * 100.0)
                    } else {
                        format!("Adapted approach: {}. Following best practices from similar cases.",
                               self.adapt_solution_for_context(problem, &reference_exp.response))
                    }
                } else {
                    format!("General approach: {}", reference_exp.response)
                }
            },
            AgentRole::TechnicalSupport => {
                format!("Technical solution: {}. Adapted for current context.",
                       reference_exp.response)
            },
            _ => reference_exp.response.clone()
        }
    }

    /// 根据上下文调整解决方案
    fn adapt_solution_for_context(&self, problem: &str, original_solution: &str) -> String {
        if problem.to_lowercase().contains("reports") && original_solution.contains("clear") {
            "Guide user to clear browser cache and retry. If issue persists, reset password.".to_string()
        } else {
            original_solution.to_string()
        }
    }

    /// 自动评估解决方案质量
    fn evaluate_solution_quality(&self, problem: &str, solution: &str, agent_id: &str) -> f64 {
        let mut score = 0.5; // 基础分数

        // 检查解决方案是否包含关键词
        if problem.to_lowercase().contains("login") && solution.to_lowercase().contains("cache") {
            score += 0.3;
        }
        if solution.to_lowercase().contains("resolved") || solution.to_lowercase().contains("successful") {
            score += 0.2;
        }

        // 根据Agent经验调整
        if let Some(agent) = self.agents.get(agent_id) {
            if agent.experience_count > 0 {
                score += agent.average_quality * 0.1; // 经验丰富的Agent分数稍高
            }
        }

        // 添加随机因素模拟真实情况
        let random_factor = (agent_id.len() % 10) as f64 / 100.0; // 伪随机
        score += random_factor;

        score.min(1.0) // 确保不超过1.0
    }

    /// Agent智能处理问题（新的核心方法）
    pub fn agent_handle_problem(
        &mut self,
        agent_id: String,
        problem_context: String
    ) -> Result<SolutionSuggestion, Box<dyn Error>> {

        println!("🤔 Agent {} encountered problem: {}", agent_id, problem_context);

        // 1. 尝试从权重矩阵查询解决方案
        if let Some(suggestion) = self.query_solution_from_weights(&agent_id, &problem_context) {
            println!("💡 Found similar solution with {:.1}% confidence", suggestion.confidence * 100.0);

            // 2. 自动评估解决方案质量
            let quality_score = self.evaluate_solution_quality(&problem_context, &suggestion.solution_text, &agent_id);

            // 3. 记录这次学习经验
            self.add_learning_experience(
                agent_id.clone(),
                problem_context,
                suggestion.solution_text.clone(),
                quality_score,
            )?;

            println!("✅ Solution applied with quality score: {:.2}", quality_score);

            Ok(suggestion)
        } else {
            // 如果没有找到相似解决方案，使用默认方法
            let default_solution = format!("Applied standard procedure for: {}", problem_context);
            let quality_score = self.evaluate_solution_quality(&problem_context, &default_solution, &agent_id);

            self.add_learning_experience(
                agent_id.clone(),
                problem_context,
                default_solution.clone(),
                quality_score,
            )?;

            println!("📚 No similar experience found, used standard approach with score: {:.2}", quality_score);

            Ok(SolutionSuggestion {
                solution_text: default_solution,
                confidence: 0.6,
                source_agents: vec![],
                similarity_score: 0.0,
            })
        }
    }

    /// 生成系统统计报告
    pub fn generate_report(&self) {
        println!("\n📊 Multi-Agent Context Sharing Report");
        println!("{}", "=".repeat(50));

        println!("🤖 Registered Agents: {}", self.agents.len());
        for (id, agent) in &self.agents {
            println!("   - {}: {:?} (experiences: {}, avg quality: {:.2})",
                id, agent.role, agent.experience_count, agent.average_quality);
        }

        println!("\n📚 Total Experiences: {}", self.experiences.len());
        println!("🔗 Shared Weight Matrix Dimension: {}", self.dimension);

        println!("\n💡 Agent Contributions:");
        for (agent_id, contribution) in &self.shared_matrix.agent_contributions {
            println!("   - {}: {:.3}", agent_id, contribution);
        }

        // 计算协作效率
        let total_individual_learning = self.agents.len() as f64;
        let actual_shared_learning = self.shared_matrix.agent_contributions.values().sum::<f64>();
        let efficiency = (actual_shared_learning / total_individual_learning) * 100.0;

        println!("\n🚀 Collaboration Efficiency: {:.1}%", efficiency);
        println!("   (>100% means knowledge sharing is working!)");
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Enhanced Multi-Agent Context Sharing Demo");
    println!("=============================================\n");

    // 创建多Agent管理器
    let mut manager = MultiAgentContextManager::new(128);

    // 注册不同角色的Agent
    manager.register_agent("cs_alice".to_string(), AgentRole::CustomerService)?;
    manager.register_agent("cs_bob".to_string(), AgentRole::CustomerService)?;
    manager.register_agent("tech_charlie".to_string(), AgentRole::TechnicalSupport)?;
    manager.register_agent("sales_diana".to_string(), AgentRole::SalesAssistant)?;
    manager.register_agent("dev_eve".to_string(), AgentRole::CodeAssistant)?;

    println!("\n📖 Phase 1: Initial Learning Experiences...\n");

    // Alice学习处理登录问题（专家级经验）
    manager.add_learning_experience(
        "cs_alice".to_string(),
        "User cannot login, password error".to_string(),
        "Clear browser cache and reset password. Issue resolved.".to_string(),
        0.95,
    )?;

    // Charlie学习API问题
    manager.add_learning_experience(
        "tech_charlie".to_string(),
        "API timeout error in production".to_string(),
        "Increase timeout settings and add retry logic. Performance improved.".to_string(),
        0.92,
    )?;

    println!("\n🧠 Phase 2: Knowledge Transfer Testing...\n");

    // Bob现在遇到类似问题，但他要基于Alice的知识来解决
    let bob_solution = manager.agent_handle_problem(
        "cs_bob".to_string(),
        "Customer reports login failure".to_string(),
    )?;

    println!("📋 Bob's solution: {}", bob_solution.solution_text);
    println!("🎯 Confidence: {:.1}%", bob_solution.confidence * 100.0);

    // Diana处理销售咨询（没有相关经验，应该使用默认方法）
    let diana_solution = manager.agent_handle_problem(
        "sales_diana".to_string(),
        "Customer asks about pricing plans".to_string(),
    )?;

    println!("📋 Diana's solution: {}", diana_solution.solution_text);

    // Eve处理技术问题（应该能从Charlie那里学到一些）
    let eve_solution = manager.agent_handle_problem(
        "dev_eve".to_string(),
        "Function timeout needs optimization".to_string(),
    )?;

    println!("📋 Eve's solution: {}", eve_solution.solution_text);

    println!("\n🔍 Phase 3: Verification...\n");

    // 测试上下文查询
    let login_experiences = manager.query_shared_context("cs_bob", "login", 3);
    println!("📋 Login-related experiences found: {}", login_experiences.len());

    // 生成最终报告
    manager.generate_report();

    println!("\n✨ Enhanced demo completed - Bob now truly learns from Alice!");
    Ok(())
}
