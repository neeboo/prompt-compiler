#!/usr/bin/env python3
"""
统一测试配置
"""

# 测试参数配置
TEST_CONFIG = {
    "conversation_rounds": 20,  # 统一使用20轮对话
    "base_url": "http://localhost:3000",
    "model": "gpt-3.5-turbo",
    "temperature": 0.7,
    "max_tokens": 150,
    "delay_between_requests": 0.1  # 请求间隔
}

# 单agent测试场景
SINGLE_AGENT_SCENARIOS = {
    "technical_consultation": [
        "我需要搭建一个高并发的电商系统，用户量预计100万",
        "系统需要支持哪些核心功能？",
        "数据库应该选择什么架构？",
        "如何设计用户认证和授权？",
        "商品搜索功能如何优化？",
        "订单处理流程怎么设计？",
        "支付系统的安全性如何保证？",
        "库存管理有什么最佳实践？",
        "如何处理大促时的流量洪峰？",
        "系统监控和日志应该怎么做？",
        "数据备份和灾难恢复策略？",
        "微服务拆分的原则是什么？",
        "API网关的作用和选型？",
        "缓存策略如何设计？",
        "数据库分库分表的方案？",
        "消息队列的使用场景？",
        "容器化部署的最佳实践？",
        "CI/CD流程如何搭建？",
        "性能测试应该关注什么指标？",
        "上线后的运维注意事项？"
    ]
}

# 多agent测试场景（3个角色，20轮对话）
MULTI_AGENT_SCENARIOS = {
    "agents": {
        "sales_manager": {
            "name": "销售经理",
            "role": "负责客户需求收集和方案推销"
        },
        "tech_lead": {
            "name": "技术负责人",
            "role": "负责技术方案设计和可行性评估"
        },
        "project_manager": {
            "name": "项目经理",
            "role": "负责项目规划和资源协调"
        }
    },
    "enterprise_project_20rounds": [
        {"agent": "sales_manager", "message": "我们有一个新的企业客户，需要一个定制的CRM系统。客户是一家500人的制造企业，年营收5亿。"},
        {"agent": "tech_lead", "message": "了解。这个CRM系统主要需要哪些功能模块？数据量大概是什么规模？"},
        {"agent": "project_manager", "message": "从项目管理角度，我们需要确定时间线。客户期望的交付时间是什么时候？"},
        {"agent": "sales_manager", "message": "客户主要需要客户管理、销售管道、报表分析和移动端支持。预计6个月内交付，预算在200-300万。"},
        {"agent": "tech_lead", "message": "6个月时间比较紧张。建议采用微服务架构，可以并行开发。数据库考虑用PostgreSQL，缓存用Redis。"},
        {"agent": "project_manager", "message": "我来制定详细的项目计划。需要多少开发人员？前端和后端分别需要几人？"},
        {"agent": "tech_lead", "message": "建议4个后端开发，2个前端开发，1个UI设计师。还需要1个DevOps工程师负责部署。"},
        {"agent": "sales_manager", "message": "成本方面客户比较敏感。我们能不能优化一下人员配置，控制在预算范围内？"},
        {"agent": "project_manager", "message": "可以考虑分阶段交付。第一阶段先做核心功能，后续迭代添加高级特性。"},
        {"agent": "tech_lead", "message": "同意分阶段方案。第一阶段可以减少到3个后端，1个前端，共享UI设计师。"},
        {"agent": "sales_manager", "message": "很好。客户还关心数据安全和隐私保护。这方面有什么方案？"},
        {"agent": "tech_lead", "message": "我们可以实施端到端加密，数据库加密存储，还有完善的权限管理系统。符合ISO27001标准。"},
        {"agent": "project_manager", "message": "安全测试也要纳入项目计划。建议找第三方安全公司做渗透测试。"},
        {"agent": "sales_manager", "message": "客户询问是否支持集成他们现有的ERP系统。这个技术上可行吗？"},
        {"agent": "tech_lead", "message": "可以通过API集成。需要了解他们ERP系统的具体类型和版本，比如SAP还是用友。"},
        {"agent": "project_manager", "message": "ERP集成会增加项目复杂度。建议作为第二阶段的功能，或者单独报价。"},
        {"agent": "sales_manager", "message": "明白了。我整理一下方案给客户。大家还有其他需要考虑的点吗？"},
        {"agent": "tech_lead", "message": "建议加上系统性能要求的讨论。比如并发用户数、响应时间等指标。"},
        {"agent": "project_manager", "message": "还有运维和培训。交付后的技术支持和用户培训也要规划好。"},
        {"agent": "sales_manager", "message": "非常全面。我会把这些都整理到提案中，包括分期付款方案。谢谢大家的配合！"}
    ]
}

# 测试结果评估标准
EVALUATION_CRITERIA = {
    "token_efficiency_thresholds": {
        "excellent": 30,  # 30%以上节省为优秀
        "good": 15,       # 15-30%节省为良好
        "fair": 5         # 5-15%节省为一般
    },
    "response_time_acceptable_increase": 200,  # 200%以内的响应时间增加可接受
    "cost_savings_target": 20  # 目标成本节省20%
}
