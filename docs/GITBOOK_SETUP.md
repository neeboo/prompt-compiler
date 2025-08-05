# GitBook 部署指南 📖

## 🎯 概述

您的 Prompt Compiler 项目现在已经完全支持 GitBook！所有文档已经配置为专业的在线文档格式，并支持**GitHub Actions自动部署**。

## 🚀 自动部署 (推荐)

### GitHub Actions + GitHub Pages

我已经为您配置了完全自动化的部署流程：

1. **推送触发自动部署**
   ```bash
   git add docs/
   git commit -m "📖 Update documentation"
   git push origin main
   ```

2. **GitHub Actions自动执行**
   - 安装GitBook CLI
   - 构建文档网站
   - 部署到GitHub Pages

3. **访问您的在线文档**
   ```
   https://your-username.github.io/prompt-compiler
   ```

### 🔧 启用GitHub Pages

在您的GitHub仓库中：
1. 进入 **Settings** → **Pages**
2. Source 选择 **Deploy from a branch**
3. Branch 选择 **gh-pages**
4. 保存设置

### ✨ 自动化特性

- **触发条件**: 推送到main分支且docs目录有变化
- **PR预览**: Pull Request会自动评论预览链接
- **缓存优化**: Node.js依赖缓存，构建更快
- **错误处理**: 构建失败会在Actions中显示详细日志

## 🖥️ 本地开发 (可选)

如果需要本地预览：

### 1. 安装 GitBook CLI

```bash
# 全局安装 GitBook 命令行工具
npm install -g gitbook-cli
```

### 2. 启动本地 GitBook 服务

```bash
# 运行一键部署脚本
./gitbook-serve.sh
```

服务启动后，访问 `http://localhost:4000` 即可查看您的文档网站。

## 📁 GitBook 文件结构

完整的 GitBook 配置包括：

```
.github/workflows/
└── gitbook.yml       # GitHub Actions工作流

docs/
├── README.md          # GitBook 首页
├── SUMMARY.md         # 目录结构 (最重要)
├── book.json          # GitBook 配置文件 (已优化CI)
├── configuration.md   # 配置指南
└── use_cases/
    ├── real_world_integration_guide.md     # 英文集成指南
    ├── real_world_integration_guide.cn.md  # 中文集成指南
    ├── multi_agent_sharing.md              # 英文多Agent指南
    └── multi_agent_sharing.cn.md           # 中文多Agent指南
```

## 🌟 GitHub Actions 工作流特性

- **智能触发**: 只在docs目录变化时构建
- **并行处理**: 支持PR预览和主分支部署
- **缓存优化**: Node.js依赖缓存
- **错误恢复**: 自动重试和详细日志

## 📖 使用流程

### 日常文档更新

1. **编辑文档**
   ```bash
   # 编辑 docs/ 目录下的文件
   vim docs/use_cases/new_feature.md
   ```

2. **更新目录** (如果添加新页面)
   ```bash
   # 编辑 docs/SUMMARY.md
   echo "* [New Feature](use_cases/new_feature.md)" >> docs/SUMMARY.md
   ```

3. **推送并自动部署**
   ```bash
   git add docs/
   git commit -m "📖 Add new feature documentation"
   git push origin main
   ```

4. **等待部署完成** (通常2-3分钟)
   访问 GitHub Actions 页面查看构建状态

### 添加新语言版本

1. 创建新的语言文件 (如 `.ja.md` 日文版)
2. 在 `docs/SUMMARY.md` 中添加链接
3. 推送即可自动部署

## 🔧 高级配置

### 自定义域名

1. 在 `docs/` 目录创建 `CNAME` 文件
   ```bash
   echo "docs.your-domain.com" > docs/CNAME
   ```

2. GitHub Actions会自动复制到构建输出

### 修改主题样式

编辑 `docs/book.json` 中的 `pluginsConfig` 部分。

## ✅ 优势总结

相比手动部署，GitHub Actions带来：

- **零维护**: 推送即部署，无需手动操作
- **版本控制**: 每次部署都有完整的git历史
- **免费托管**: 利用GitHub Pages免费服务
- **高可用**: GitHub的全球CDN加速
- **团队协作**: 支持多人同时维护文档

您的 Prompt Compiler 文档现在具备了企业级的自动化部署能力！🎉
