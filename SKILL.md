---
name: external-memory
description: "外部记忆系统 — 持久化记忆存储/检索/梦境引擎"
metadata:
  openclaw:
    emoji: 🧠
    requires:
      bins: ["python3"]
---

# External Memory System

你有一个独立的外部记忆系统运行在 `localhost:50051`（gRPC）。

它不同于 MEMORY.md 文件记忆——**这个记忆是持久化、可搜索、不会丢的**。

## 基本命令

### 存储重要信息

```bash
# 保存一条记忆
memory-cli store <agent_id> shared "<content>"

# 保存到项目空间
memory-cli store <agent_id> "project/ai-drama/*" "<content>"
```

### 搜索记忆

```bash
memory-cli search <agent_id> "<query>" --limit 10 --ns shared
```

返回 JSON 数组，每条含 `id, importance, content, namespace, created_at`。

### 获取项目上下文

```bash
memory-cli context ai-drama <agent_id>
```

### 手动触发梦境（总结→推理）

```bash
memory-cli dream <agent_id> --ns shared
```

## 什么时候用

| 情况 | 用外部记忆 | 用 MEMORY.md |
|------|-----------|-------------|
| 日常对话日志 | ✅ | ❌ |
| 跨 session 的重要决策 | ✅ | ✅ |
| 技术配置/API Key | ❌ | ✅（安全） |
| 项目进展追踪 | ✅ | ✅ |
| 频繁检索的事实 | ✅ | ❌ |
| 个人情感/关系记录 | ❌ | ✅（隐私） |

**原则**：外部记忆存"数据"，MEMORY.md 存"故事"。
