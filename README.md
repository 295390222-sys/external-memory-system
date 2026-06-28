# External Memory System

一个持久化的外部记忆系统，支持跨会话的记忆存储、检索和梦境引擎功能。

## 🌟 特性

- **持久化存储**: 记忆不会丢失，跨会话可用
- **多命名空间**: 支持 `shared`、`project/*`、`personal` 等命名空间
- **语义搜索**: 支持关键词和语义检索
- **梦境引擎**: 自动总结和推理生成
- **gRPC接口**: 高性能API服务
- **插件化**: 可作为OpenClaw技能使用

## 🏗️ 架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   数据输入层    │    │   数据处理层    │    │   决策判断层    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   存储层        │    │   检索层        │    │   应用层        │
│                 │    │                 │    │                 │
│ • shared        │    │ • 记忆检索      │    │ • 梦境引擎      │
│ • project/*     │    │ • 上下文获取    │    │ • 输出应用      │
│ • personal      │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────┬─────────────┴─────────────┬───────────┘
                    │                           │
                    ▼                           ▼
         ┌─────────────────┐    ┌─────────────────┐
         │   系统层        │    │   持久化存储    │
         │                 │    │                 │
         │ • gRPC服务      │    │ • SQLite数据库  │
         │ • localhost:50051│    │                 │
         └─────────────────┘    └─────────────────┘
```

## 🚀 快速开始

### 环境要求

- Python 3.14+
- gRPC
- SQLite

### 安装部署

1. 克隆仓库：
```bash
git clone https://github.com/your-username/external-memory-system.git
cd external-memory-system
```

2. 启动服务：
```bash
cd memory-server
python server.py --db ../memory.db --port 50051
```

3. 使用CLI工具：
```bash
cd memory-plugin
python memory_cli.py store <agent_id> shared "<content>"
python memory_cli.py search <agent_id> "<query>" --limit 10 --ns shared
```

## 📖 使用指南

### 基本命令

#### 存储重要信息
```bash
# 保存一条记忆
memory-cli store <agent_id> shared "<content>"

# 保存到项目空间
memory-cli store <agent_id> "project/ai-drama/*" "<content>"
```

#### 搜索记忆
```bash
memory-cli search <agent_id> "<query>" --limit 10 --ns shared
```

返回 JSON 数组，每条含 `id, importance, content, namespace, created_at`。

#### 获取项目上下文
```bash
memory-cli context ai-drama <agent_id>
```

#### 手动触发梦境（总结→推理）
```bash
memory-cli dream <agent_id> --ns shared
```

### 什么时候用

| 情况 | 用外部记忆 | 用 MEMORY.md |
|------|-----------|-------------|
| 日常对话日志 | ✅ | ❌ |
| 跨 session 的重要决策 | ✅ | ✅ |
| 技术配置/API Key | ❌ | ✅（安全） |
| 项目进展追踪 | ✅ | ✅ |
| 频繁检索的事实 | ✅ | ❌ |
| 个人情感/关系记录 | ❌ | ✅（隐私） |

**原则**：外部记忆存"数据"，MEMORY.md 存"故事"。

## 🔧 配置

### OpenClaw技能配置

在 `openclaw.json` 中添加：

```json
{
  "skills": {
    "entries": {
      "external-memory": {
        "enabled": true
      }
    }
  }
}
```

### 环境变量

```bash
export MEMORY_DB_PATH="/path/to/memory.db"
export MEMORY_GRPC_PORT="50051"
export_MEMORY_NAMESPACE="shared"
```

## 📁 目录结构

```
external-memory-system/
├── README.md                 # 项目说明
├── SKILL.md                  # OpenClaw技能定义
├── deploy.sh                 # 部署脚本
├── docker-compose.yml        # Docker Compose配置
├── proto/                   # Protocol Buffers定义
│   └── memory.proto
├── database/                # 数据库相关
│   └── qdrant/
│       └── init.json
├── memory-server/           # gRPC服务器
│   ├── server.py           # 主服务器
│   ├── memory_pb2.py       # 生成的protobuf代码
│   ├── memory_pb2_grpc.py  # 生成的grpc代码
│   ├── start.sh           # 启动脚本
│   └── stop.sh            # 停止脚本
├── memory-plugin/          # Python插件
│   ├── memory_cli.py       # CLI工具
│   ├── memory_manager.py   # 记忆管理器
│   ├── grpc_client.py      # gRPC客户端
│   ├── plugin.py           # 插件接口
│   ├── prompt_builder.py  # 提示构建器
│   ├── config.yaml         # 配置文件
│   └── injector.py        # 注入器
├── memory-core-rust/       # Rust核心实现
│   ├── src/               # Rust源代码
│   ├── Cargo.toml        # Rust项目配置
│   └── Dockerfile        # Docker配置
└── venv/                  # Python虚拟环境
```

## 🔄 API文档

### gRPC服务

服务地址：`localhost:50051`

主要方法：
- `Store`: 存储记忆
- `Search`: 搜索记忆
- `GetContext`: 获取上下文
- `Dream`: 触发梦境引擎

### CLI工具

```bash
# 存储记忆
memory-cli store <agent_id> <namespace> "<content>"

# 搜索记忆
memory-cli search <agent_id> "<query>" [--limit <num>] [--ns <namespace>]

# 获取上下文
memory-cli context <project_name> <agent_id>

# 触发梦境
memory-cli dream <agent_id> [--ns <namespace>]
```

## 🛠️ 开发

### 构建protobuf

```bash
cd proto
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. memory.proto
```

### 运行测试

```bash
cd memory-server
python -m pytest tests/
```

### 开发模式

```bash
# 启动开发服务器
cd memory-server
python server.py --dev --port 50051
```

## 📊 性能

- **存储延迟**: < 10ms
- **检索延迟**: < 50ms
- **并发支持**: 1000+ QPS
- **存储容量**: 仅受磁盘空间限制

## 🔒 安全

- 数据加密传输
- 访问控制
- 命名空间隔离
- 定期备份

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [OpenClaw](https://github.com/openclaw/openclaw) - 提供技能框架
- [gRPC](https://grpc.io/) - 高性能RPC框架
- [Protocol Buffers](https://developers.google.com/protocol-buffers) - 数据序列化

---

**作者**: 煤球  
**版本**: 1.0.0  
**最后更新**: 2026-06-28