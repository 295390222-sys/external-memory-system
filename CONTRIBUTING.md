# 贡献指南

感谢您对外部记忆系统的关注！我们欢迎各种形式的贡献。

## 🤝 如何贡献

### 报告问题

如果您发现了bug或有改进建议，请：

1. 检查是否已有相关的Issue
2. 创建新的Issue，包含：
   - 详细的问题描述
   - 重现步骤
   - 期望的行为
   - 实际的行为
   - 环境信息（操作系统、Python版本等）

### 功能请求

我们欢迎新的功能建议！请创建一个Issue描述：

- 功能的详细描述
- 使用场景
- 实现建议
- 相关的Issue链接

### 代码贡献

1. **Fork 仓库**
2. **创建特性分支**：
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **进行开发**：
   - 遵循现有的代码风格
   - 添加适当的测试
   - 更新文档
4. **提交更改**：
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```
5. **推送分支**：
   ```bash
   git push origin feature/your-feature-name
   ```
6. **创建Pull Request**

## 📝 代码规范

### Python代码

- 遵循PEP 8规范
- 使用4个空格缩进
- 最大行长度：88字符
- 使用类型注解
- 编写docstring

### Rust代码

- 遵循Rust官方规范
- 使用`cargo fmt`格式化
- 编写测试用例

### 提交信息

使用常规提交格式：

```
<type>(<scope>): <description>

<body>

<footer>
```

类型：
- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `style`: 代码格式化
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建或工具变动

## 🧪 测试

### 运行测试

```bash
# Python测试
cd memory-server
python -m pytest tests/

# Rust测试
cd memory-core-rust
cargo test

# 集成测试
cd memory-plugin
python test_integration.py
```

### 添加测试

- 为新功能添加单元测试
- 为关键路径添加集成测试
- 使用mock对象隔离依赖

## 📖 文档

### 更新文档

- 修改功能时更新相关文档
- 添加新功能时添加使用示例
- 保持文档的准确性

### 文档类型

- **README.md**: 项目概述和快速开始
- **SKILL.md**: OpenClaw技能配置
- **API文档**: gRPC接口说明
- **部署指南**: 安装和配置说明

## 🔄 Pull Request流程

1. **PR描述**：
   - 清晰描述变更内容
   - 解释变更的原因
   - 列出相关的Issue

2. **检查清单**：
   - [ ] 代码已格式化
   - [ ] 测试通过
   - [ ] 文档已更新
   - [ ] CI检查通过
   - [ ] 无安全漏洞

3. **审查流程**：
   - 等待代码审查
   - 根据反馈进行修改
   - 确认所有检查通过

## 🏷️ 版本管理

我们使用语义化版本控制：

- **主版本号**: 不兼容的API修改
- **次版本号**: 向下兼容的功能性新增
- **修订号**: 向下兼容的问题修正

## 📞 联系方式

- 提交Issue
- 参与讨论
- 邮件联系（如有需要）

---

感谢您的贡献！🎉