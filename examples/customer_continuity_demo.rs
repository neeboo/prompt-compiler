use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// 客户基本信息
#[derive(Debug, Clone)]
pub struct CustomerProfile {
    pub name: String,
    pub age: u32,
    pub education: String,
    pub income_level: String,
    pub communication_style: String, // 技术型、简洁型、详细型
    pub preferred_channel: String,   // 电话、邮件、在线聊天
}

/// 客户互动历史
#[derive(Debug, Clone)]
pub struct CustomerInteraction {
    pub interaction_id: String,
    pub customer_name: String,
    pub agent_id: String,
    pub timestamp: u64,
    pub problem_category: String,
    pub problem_description: String,
    pub solution_provided: String,
    pub customer_satisfaction: f64,
    pub resolution_status: String, // 已解决、部分解决、未解决、需要跟进
    pub follow_up_needed: bool,
    pub notes: String,
}

/// 客服Agent信息
#[derive(Debug, Clone)]
pub struct CustomerServiceAgent {
    pub agent_id: String,
    pub name: String,
    pub specialties: Vec<String>, // 技术支持、账单、产品咨询等
    pub experience_level: String, // 初级、中级、高级
    pub current_workload: u32,
}

/// 智能客服上下文管理系统
pub struct CustomerContextManager {
    pub customers: HashMap<String, CustomerProfile>,
    pub interactions: Vec<CustomerInteraction>,
    pub agents: HashMap<String, CustomerServiceAgent>,
    pub shared_knowledge: HashMap<String, Vec<f64>>, // 客户名称 -> 语义向量
    pub problem_patterns: HashMap<String, Vec<f64>>, // 问题类型 -> 语义向量
}

impl CustomerContextManager {
    pub fn new() -> Self {
        Self {
            customers: HashMap::new(),
            interactions: Vec::new(),
            agents: HashMap::new(),
            shared_knowledge: HashMap::new(),
            problem_patterns: HashMap::new(),
        }
    }

    /// 注册客户
    pub fn register_customer(&mut self, profile: CustomerProfile) -> Result<(), Box<dyn Error>> {
        let customer_vector = self.generate_customer_vector(&profile);
        self.shared_knowledge.insert(profile.name.clone(), customer_vector);
        self.customers.insert(profile.name.clone(), profile.clone());

        println!("👤 Customer {} registered: {} years old, {} education, {} income",
                profile.name, profile.age, profile.education, profile.income_level);
        Ok(())
    }

    /// 注册客服Agent
    pub fn register_agent(&mut self, agent: CustomerServiceAgent) -> Result<(), Box<dyn Error>> {
        self.agents.insert(agent.agent_id.clone(), agent.clone());
        println!("👨‍💼 Agent {} ({}) registered - specialties: {:?}",
                agent.agent_id, agent.name, agent.specialties);
        Ok(())
    }

    /// 客户发起新的问题咨询
    pub fn customer_inquiry(
        &mut self,
        customer_name: String,
        agent_id: String,
        problem_category: String,
        problem_description: String,
    ) -> Result<String, Box<dyn Error>> {

        println!("\n📞 New inquiry from customer: {}", customer_name);
        println!("📋 Problem: {} - {}", problem_category, problem_description);

        // 1. 获取客户历史上下文
        let customer_context = self.get_customer_context(&customer_name);

        // 2. 智能生成解决方案
        let solution = self.generate_intelligent_solution(
            &customer_name,
            &agent_id,
            &problem_category,
            &problem_description,
            &customer_context,
        );

        // 3. 评估解决方案质量
        let satisfaction = self.evaluate_solution_quality(
            &customer_name,
            &problem_category,
            &solution,
        );

        // 4. 记录这次互动
        let interaction = CustomerInteraction {
            interaction_id: format!("int_{}_{}", customer_name, self.interactions.len()),
            customer_name: customer_name.clone(),
            agent_id: agent_id.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            problem_category: problem_category.clone(),
            problem_description: problem_description.clone(),
            solution_provided: solution.clone(),
            customer_satisfaction: satisfaction,
            resolution_status: if satisfaction > 0.8 { "已解决".to_string() }
                              else if satisfaction > 0.6 { "部分解决".to_string() }
                              else { "需要跟进".to_string() },
            follow_up_needed: satisfaction < 0.7,
            notes: "".to_string(),
        };

        self.interactions.push(interaction);

        // 5. 更新共享知识库
        self.update_shared_knowledge(&customer_name, &problem_category, &solution, satisfaction);

        println!("✅ Agent {} provided solution (satisfaction: {:.1}%)", agent_id, satisfaction * 100.0);
        println!("💬 Solution: {}", solution);

        Ok(solution)
    }

    /// 获取客户完整上下文
    fn get_customer_context(&self, customer_name: &str) -> String {
        let mut context = String::new();

        // 客户基本信息
        if let Some(profile) = self.customers.get(customer_name) {
            context.push_str(&format!(
                "客户档案: {} ({}岁, {}, {}收入, 偏好{}沟通)\n",
                profile.name, profile.age, profile.education,
                profile.income_level, profile.communication_style
            ));
        }

        // 历史问题记录
        let customer_interactions: Vec<&CustomerInteraction> = self.interactions.iter()
            .filter(|interaction| interaction.customer_name == customer_name)
            .collect();

        if !customer_interactions.is_empty() {
            context.push_str("历史记录:\n");
            for (i, interaction) in customer_interactions.iter().rev().take(3).enumerate() {
                context.push_str(&format!(
                    "  {}. {} - {} ({}%) - Agent: {}\n",
                    i + 1,
                    interaction.problem_category,
                    interaction.resolution_status,
                    (interaction.customer_satisfaction * 100.0) as u32,
                    interaction.agent_id
                ));
            }
        }

        context
    }

    /// 智能生成解决方案
    fn generate_intelligent_solution(
        &self,
        customer_name: &str,
        agent_id: &str,
        problem_category: &str,
        problem_description: &str,
        customer_context: &str,
    ) -> String {

        // 获取客户和Agent信息
        let customer = self.customers.get(customer_name);
        let agent = self.agents.get(agent_id);

        // 查找相似历史问题
        let similar_solutions = self.find_similar_solutions(customer_name, problem_category);

        // 根据客户背景和历史记录生成个性化解决方案
        match problem_category {
            "登录问题" => self.generate_login_solution(customer, agent, &similar_solutions),
            "账单查询" => self.generate_billing_solution(customer, agent, &similar_solutions),
            "产品咨询" => self.generate_product_solution(customer, agent, &similar_solutions),
            "技术支持" => self.generate_technical_solution(customer, agent, &similar_solutions),
            "投诉处理" => self.generate_complaint_solution(customer, agent, &similar_solutions),
            _ => format!("根据您的情况，我为您提供标准解决方案。")
        }
    }

    /// 生成登录问题解决方案
    fn generate_login_solution(
        &self,
        customer: Option<&CustomerProfile>,
        agent: Option<&CustomerServiceAgent>,
        similar_solutions: &[String],
    ) -> String {
        let mut solution = String::new();

        if let Some(profile) = customer {
            // 根据客户技术水平调整说明详细程度
            match profile.education.as_str() {
                "本科以上" | "研究生" => {
                    solution.push_str("技术方案: ");
                    if !similar_solutions.is_empty() {
                        solution.push_str(&format!("基于您之前的问题记录，建议: {}。", similar_solutions[0]));
                    } else {
                        solution.push_str("1) 清除浏览器缓存和Cookie 2) 检查网络连接 3) 尝试无痕模式登录。");
                    }
                },
                _ => {
                    solution.push_str("简化步骤: ");
                    if !similar_solutions.is_empty() {
                        solution.push_str(&format!("您上次遇到类似问题，我们这样解决的: {}。", similar_solutions[0]));
                    } else {
                        solution.push_str("请按以下步骤操作: 1) 关闭浏览器重新打开 2) 重新输入用户名密码 3) 如还有问题请联系我们。");
                    }
                }
            }

            // 根据沟通风格调整语气
            match profile.communication_style.as_str() {
                "技术型" => solution.push_str(" 技术细节可随时咨询。"),
                "简洁型" => solution.push_str(" 如需帮助请告知。"),
                _ => solution.push_str(" 我会持续跟进直到问题解决。")
            }
        }

        solution
    }

    /// 生成账单查询解决方案
    fn generate_billing_solution(
        &self,
        customer: Option<&CustomerProfile>,
        agent: Option<&CustomerServiceAgent>,
        similar_solutions: &[String],
    ) -> String {
        if let Some(profile) = customer {
            match profile.income_level.as_str() {
                "高收入" => "为您查询详细账单明细，包括所有服务项目和优惠信息。如需发票请提供邮箱地址。".to_string(),
                "中等收入" => "为您核查本月账单，如有疑问可申请分期付款或优惠方案。".to_string(),
                _ => "为您查询账单信息，我们有多种付款方式和优惠政策可供选择。".to_string()
            }
        } else {
            "为您查询账单详情。".to_string()
        }
    }

    /// 生成产品咨询解决方案
    fn generate_product_solution(
        &self,
        customer: Option<&CustomerProfile>,
        agent: Option<&CustomerServiceAgent>,
        similar_solutions: &[String],
    ) -> String {
        if let Some(profile) = customer {
            let age_group = if profile.age < 30 { "年轻用户" }
                           else if profile.age < 50 { "中年用户" }
                           else { "成熟用户" };

            format!("根据您的{}特点和{}背景，推荐适合的产品方案。", age_group, profile.education)
        } else {
            "为您推荐合适的产品。".to_string()
        }
    }

    /// 生成技术支持解决方案
    fn generate_technical_solution(
        &self,
        customer: Option<&CustomerProfile>,
        agent: Option<&CustomerServiceAgent>,
        similar_solutions: &[String],
    ) -> String {
        let mut solution = String::new();

        if !similar_solutions.is_empty() {
            solution.push_str(&format!("基于您的历史记录，上次类似问题的解决方案是: {}。", similar_solutions[0]));
        }

        if let Some(agent_info) = agent {
            if agent_info.specialties.contains(&"技术支持".to_string()) {
                solution.push_str(" 我是技术专家，可以为您提供深度技术支持。");
            }
        }

        solution.push_str(" 如问题复杂，我会安排技术专家为您服务。");
        solution
    }

    /// 生成投诉处理解决方案
    fn generate_complaint_solution(
        &self,
        customer: Option<&CustomerProfile>,
        agent: Option<&CustomerServiceAgent>,
        similar_solutions: &[String],
    ) -> String {
        let mut solution = String::new();

        if let Some(profile) = customer {
            // 检查客户历史满意度
            let avg_satisfaction = self.get_customer_avg_satisfaction(&profile.name);

            if avg_satisfaction < 0.6 {
                solution.push_str("我注意到您之前也有一些困扰，我会特别重视您的问题。");
            }

            solution.push_str("我会立即升级处理您的投诉，并在24小时内给您明确回复。同时为您申请适当的补偿方案。");
        }

        solution
    }

    /// 查找相似解决方案
    fn find_similar_solutions(&self, customer_name: &str, problem_category: &str) -> Vec<String> {
        self.interactions.iter()
            .filter(|interaction|
                interaction.customer_name == customer_name &&
                interaction.problem_category == problem_category &&
                interaction.customer_satisfaction > 0.7
            )
            .map(|interaction| interaction.solution_provided.clone())
            .collect()
    }

    /// 获取客户平均满意度
    fn get_customer_avg_satisfaction(&self, customer_name: &str) -> f64 {
        let customer_interactions: Vec<&CustomerInteraction> = self.interactions.iter()
            .filter(|interaction| interaction.customer_name == customer_name)
            .collect();

        if customer_interactions.is_empty() {
            return 0.8; // 新客户默认满意度
        }

        let total_satisfaction: f64 = customer_interactions.iter()
            .map(|interaction| interaction.customer_satisfaction)
            .sum();

        total_satisfaction / customer_interactions.len() as f64
    }

    /// 生成客户向量（简化）
    fn generate_customer_vector(&self, profile: &CustomerProfile) -> Vec<f64> {
        let mut vector = vec![0.0; 32];

        // 年龄特征
        vector[0] = profile.age as f64 / 100.0;

        // 教育背景特征
        vector[1] = match profile.education.as_str() {
            "高中" => 0.3,
            "大专" => 0.5,
            "本科" => 0.7,
            "本科以上" => 0.8,
            "研究生" => 0.9,
            _ => 0.5
        };

        // 收入水平特征
        vector[2] = match profile.income_level.as_str() {
            "低收入" => 0.3,
            "中等收入" => 0.6,
            "高收入" => 0.9,
            _ => 0.5
        };

        // 沟通风格特征
        vector[3] = match profile.communication_style.as_str() {
            "技术型" => 0.9,
            "简洁型" => 0.3,
            "详细型" => 0.7,
            _ => 0.5
        };

        vector
    }

    /// 评估解决方案质量
    fn evaluate_solution_quality(
        &self,
        _customer_name: &str,
        problem_category: &str,
        _solution: &str,
    ) -> f64 {
        let mut score: f64 = 0.6; // 基础分数，明确指定类型

        // 解决方案长度和详细程度
        if _solution.len() > 50 {
            score += 0.1;
        }

        // 个性化程度
        if _solution.contains("您的") || _solution.contains("根据您") {
            score += 0.2;
        }

        // 历史上下文利用
        if _solution.contains("上次") || _solution.contains("之前") || _solution.contains("历史") {
            score += 0.2;
        }

        // 根据问题类型调整
        match problem_category {
            "投诉处理" => score += 0.1, // 投诉处理要求更高
            "技术支持" => if _solution.contains("技术") { score += 0.1 },
            _ => {}
        }

        score.min(1.0)
    }

    /// 更新共享知识库
    fn update_shared_knowledge(
        &mut self,
        customer_name: &str,
        problem_category: &str,
        solution: &str,
        satisfaction: f64,
    ) {
        // 这里可以实现更复杂的知识更新逻辑
        if satisfaction > 0.8 {
            self.problem_patterns.entry(problem_category.to_string())
                .or_insert_with(|| vec![0.0; 16]);
        }
    }

    /// 生成客户服务报告
    pub fn generate_customer_service_report(&self) {
        println!("\n📊 Customer Service Context Management Report");
        println!("{}", "=".repeat(60));

        // 客户统计
        println!("👥 Total Customers: {}", self.customers.len());
        for (name, profile) in &self.customers {
            let avg_satisfaction = self.get_customer_avg_satisfaction(name);
            let interaction_count = self.interactions.iter()
                .filter(|i| i.customer_name == *name)
                .count();

            println!("   - {}: {} interactions, {:.1}% avg satisfaction",
                    name, interaction_count, avg_satisfaction * 100.0);
        }

        // Agent统计
        println!("\n👨‍💼 Agents Performance:");
        for (agent_id, agent) in &self.agents {
            let agent_interactions: Vec<&CustomerInteraction> = self.interactions.iter()
                .filter(|i| i.agent_id == *agent_id)
                .collect();

            if !agent_interactions.is_empty() {
                let avg_satisfaction: f64 = agent_interactions.iter()
                    .map(|i| i.customer_satisfaction)
                    .sum::<f64>() / agent_interactions.len() as f64;

                println!("   - {} ({}): {} cases, {:.1}% avg satisfaction",
                        agent_id, agent.name, agent_interactions.len(), avg_satisfaction * 100.0);
            }
        }

        // 问题类型统计
        println!("\n📋 Problem Categories:");
        let mut category_stats: HashMap<String, (u32, f64)> = HashMap::new();

        for interaction in &self.interactions {
            let entry = category_stats.entry(interaction.problem_category.clone())
                .or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += interaction.customer_satisfaction;
        }

        for (category, (count, total_satisfaction)) in category_stats {
            println!("   - {}: {} cases, {:.1}% avg satisfaction",
                    category, count, (total_satisfaction / count as f64) * 100.0);
        }

        // 上下文连续性效果
        let context_utilization = self.calculate_context_utilization();
        println!("\n🔗 Context Continuity Effectiveness: {:.1}%", context_utilization * 100.0);
        println!("   (Higher values indicate better use of customer history)");
    }

    /// 计算上下文利用率
    fn calculate_context_utilization(&self) -> f64 {
        let repeat_customers = self.interactions.iter()
            .filter(|interaction| {
                self.interactions.iter()
                    .filter(|i| i.customer_name == interaction.customer_name)
                    .count() > 1
            })
            .count();

        if self.interactions.is_empty() {
            return 0.0;
        }

        repeat_customers as f64 / self.interactions.len() as f64
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Customer Context Continuity Demo");
    println!("===================================\n");

    let mut manager = CustomerContextManager::new();

    // 注册客服人员
    manager.register_agent(CustomerServiceAgent {
        agent_id: "agent_alice".to_string(),
        name: "Alice Wang".to_string(),
        specialties: vec!["登录问题".to_string(), "账单查询".to_string()],
        experience_level: "高级".to_string(),
        current_workload: 0,
    })?;

    manager.register_agent(CustomerServiceAgent {
        agent_id: "agent_bob".to_string(),
        name: "Bob Chen".to_string(),
        specialties: vec!["技术支持".to_string(), "产品咨询".to_string()],
        experience_level: "中级".to_string(),
        current_workload: 0,
    })?;

    manager.register_agent(CustomerServiceAgent {
        agent_id: "agent_carol".to_string(),
        name: "Carol Li".to_string(),
        specialties: vec!["投诉处理".to_string(), "账单查询".to_string()],
        experience_level: "高级".to_string(),
        current_workload: 0,
    })?;

    // 注册客户
    manager.register_customer(CustomerProfile {
        name: "张先生".to_string(),
        age: 35,
        education: "本科".to_string(),
        income_level: "中等收入".to_string(),
        communication_style: "技术型".to_string(),
        preferred_channel: "在线聊天".to_string(),
    })?;

    manager.register_customer(CustomerProfile {
        name: "李女士".to_string(),
        age: 28,
        education: "研究生".to_string(),
        income_level: "高收入".to_string(),
        communication_style: "简洁型".to_string(),
        preferred_channel: "邮件".to_string(),
    })?;

    manager.register_customer(CustomerProfile {
        name: "王老师".to_string(),
        age: 55,
        education: "大专".to_string(),
        income_level: "中等收入".to_string(),
        communication_style: "详细型".to_string(),
        preferred_channel: "电话".to_string(),
    })?;

    println!("\n📅 Day 1: Initial Customer Interactions");
    println!("--------------------------------------");

    // 第一天：初始问题
    manager.customer_inquiry(
        "张先生".to_string(),
        "agent_alice".to_string(),
        "登录问题".to_string(),
        "无法登录系统，提示密码错误".to_string(),
    )?;

    manager.customer_inquiry(
        "李女士".to_string(),
        "agent_bob".to_string(),
        "产品咨询".to_string(),
        "了解高级会员服务内容".to_string(),
    )?;

    manager.customer_inquiry(
        "王老师".to_string(),
        "agent_carol".to_string(),
        "账单查询".to_string(),
        "本月账单比上月高很多".to_string(),
    )?;

    println!("\n📅 Day 2: Follow-up and New Issues");
    println!("----------------------------------");

    // 第二天：相同客户，不同Agent
    manager.customer_inquiry(
        "张先生".to_string(),
        "agent_bob".to_string(), // 不同的Agent
        "技术支持".to_string(),
        "登录后系统很卡顿".to_string(),
    )?;

    manager.customer_inquiry(
        "李女士".to_string(),
        "agent_carol".to_string(), // 不同的Agent
        "账单查询".to_string(),
        "升级会员后的费用确认".to_string(),
    )?;

    println!("\n📅 Day 3: Complaint and Resolution");
    println!("----------------------------------");

    // 第三天：投诉处理
    manager.customer_inquiry(
        "张先生".to_string(),
        "agent_carol".to_string(), // 又换了Agent
        "投诉处理".to_string(),
        "连续两天遇到技术问题，影响工作".to_string(),
    )?;

    manager.customer_inquiry(
        "王老师".to_string(),
        "agent_alice".to_string(), // 不同的Agent
        "登录问题".to_string(),
        "忘记密码，需要重置".to_string(),
    )?;

    // 生成报告
    manager.generate_customer_service_report();

    println!("\n✨ Demo completed - Customer context maintained across agents!");
    Ok(())
}
