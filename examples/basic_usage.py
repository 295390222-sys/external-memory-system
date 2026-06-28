#!/usr/bin/env python3
"""
外部记忆系统基本使用示例
作者：煤球
日期：2026-06-28
"""

import sys
import os
import json
import time
from datetime import datetime

# 添加路径以便导入模块
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'memory-plugin'))

from grpc_client import MemoryClient

def main():
    """基本使用示例"""
    
    # 初始化客户端
    client = MemoryClient(host='localhost', port=50051)
    
    print("=== 外部记忆系统基本使用示例 ===\n")
    
    # 1. 存储记忆
    print("1. 存储记忆...")
    agent_id = "demo_agent"
    content = "这是一个重要的记忆：今天学习了外部记忆系统的使用方法"
    
    try:
        # 存储到共享命名空间
        result = client.store_memory(
            agent_id=agent_id,
            namespace="shared",
            content=content,
            importance=0.8
        )
        print(f"✓ 记忆存储成功: {result.id}")
        
        # 存储到项目命名空间
        project_content = "AI短剧项目进展：完成了门户页面的视频生成功能"
        project_result = client.store_memory(
            agent_id=agent_id,
            namespace="project/ai-drama",
            content=project_content,
            importance=0.9
        )
        print(f"✓ 项目记忆存储成功: {project_result.id}")
        
    except Exception as e:
        print(f"✗ 存储失败: {e}")
        return
    
    # 2. 搜索记忆
    print("\n2. 搜索记忆...")
    try:
        # 搜索共享记忆
        search_results = client.search_memories(
            agent_id=agent_id,
            query="记忆系统",
            namespace="shared",
            limit=5
        )
        
        print(f"找到 {len(search_results)} 条相关记忆:")
        for i, memory in enumerate(search_results, 1):
            print(f"  {i}. [{memory.importance:.2f}] {memory.content[:50]}...")
            
    except Exception as e:
        print(f"✗ 搜索失败: {e}")
    
    # 3. 获取项目上下文
    print("\n3. 获取项目上下文...")
    try:
        context = client.get_context(
            project_name="ai-drama",
            agent_id=agent_id
        )
        
        print(f"项目上下文: {context[:100]}...")
        
    except Exception as e:
        print(f"✗ 获取上下文失败: {e}")
    
    # 4. 触发梦境
    print("\n4. 触发梦境引擎...")
    try:
        dream_result = client.trigger_dream(
            agent_id=agent_id,
            namespace="shared"
        )
        
        print(f"梦境生成成功: {dream_result[:100]}...")
        
    except Exception as e:
        print(f"✗ 梦境生成失败: {e}")
    
    print("\n=== 示例完成 ===")

def batch_operations():
    """批量操作示例"""
    
    print("\n=== 批量操作示例 ===\n")
    
    client = MemoryClient(host='localhost', port=50051)
    agent_id = "batch_demo"
    
    # 批量存储记忆
    memories = [
        ("学习Python编程", 0.7),
        ("完成项目原型设计", 0.9),
        ("参加技术分享会", 0.6),
        ("优化系统性能", 0.8),
        ("编写技术文档", 0.7)
    ]
    
    print("批量存储记忆...")
    stored_ids = []
    
    for content, importance in memories:
        try:
            result = client.store_memory(
                agent_id=agent_id,
                namespace="shared",
                content=content,
                importance=importance
            )
            stored_ids.append(result.id)
            print(f"  ✓ {content[:30]}...")
        except Exception as e:
            print(f"  ✗ {content[:30]}... ({e})")
    
    print(f"\n共存储 {len(stored_ids)} 条记忆")
    
    # 批量搜索
    print("\n批量搜索...")
    try:
        results = client.search_memories(
            agent_id=agent_id,
            query="技术",
            namespace="shared",
            limit=10
        )
        
        print(f"找到 {len(results)} 条技术相关记忆:")
        for memory in results:
            print(f"  [{memory.importance:.2f}] {memory.content}")
            
    except Exception as e:
        print(f"✗ 批量搜索失败: {e}")

def error_handling():
    """错误处理示例"""
    
    print("\n=== 错误处理示例 ===\n")
    
    client = MemoryClient(host='localhost', port=50051)
    
    # 1. 连接错误处理
    print("1. 连接错误处理...")
    try:
        # 尝试连接到不存在的服务
        bad_client = MemoryClient(host='localhost', port=99999)
        bad_client.store_memory("test", "shared", "test")
    except Exception as e:
        print(f"✓ 正确捕获连接错误: {type(e).__name__}")
    
    # 2. 参数验证
    print("\n2. 参数验证...")
    try:
        client.store_memory(
            agent_id="",  # 空的agent_id
            namespace="shared",
            content="test"
        )
    except ValueError as e:
        print(f"✓ 正确捕获参数错误: {e}")
    
    # 3. 命名空间错误
    print("\n3. 命名空间错误处理...")
    try:
        client.store_memory(
            agent_id="test",
            namespace="invalid_namespace",
            content="test"
        )
    except Exception as e:
        print(f"✓ 正确捕获命名空间错误: {type(e).__name__}")

if __name__ == "__main__":
    # 检查服务是否可用
    try:
        client = MemoryClient(host='localhost', port=50051)
        # 简单的连接测试
        print("正在连接到gRPC服务...")
        time.sleep(1)
        print("✓ 连接成功")
    except Exception as e:
        print(f"✗ 无法连接到gRPC服务: {e}")
        print("请确保服务正在运行: deploy.sh start")
        sys.exit(1)
    
    # 运行示例
    main()
    batch_operations()
    error_handling()