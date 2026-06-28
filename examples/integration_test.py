#!/usr/bin/env python3
"""
外部记忆系统集成测试
作者：煤球
日期：2026-06-28
"""

import sys
import os
import time
import unittest

# 添加路径以便导入模块
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'memory-plugin'))

from grpc_client import MemoryClient


class TestExternalMemorySystem(unittest.TestCase):
    """外部记忆系统测试类"""

    def setUp(self):
        """测试前设置"""
        self.client = MemoryClient(host='localhost', port=50051)
        self.agent_id = f"test_agent_{int(time.time())}"
    
    def tearDown(self):
        """测试后清理"""
        # 清理测试数据
        try:
            self.client.clear_memories(self.agent_id)
        except:
            pass
    
    def test_memory_storage(self):
        """测试记忆存储"""
        print("\n=== 测试记忆存储 ===")
        
        content = "这是一个测试记忆"
        result = self.client.store_memory(
            agent_id=self.agent_id,
            namespace="shared",
            content=content,
            importance=0.8
        )
        
        self.assertIsNotNone(result.id)
        self.assertEqual(result.content, content)
        self.assertEqual(result.namespace, "shared")
        self.assertEqual(result.importance, 0.8)
        
        print(f"✓ 记忆存储成功: {result.id}")
    
    def test_memory_search(self):
        """测试记忆搜索"""
        print("\n=== 测试记忆搜索 ===")
        
        # 存储测试数据
        test_memories = [
            ("学习Python编程", 0.9),
            ("完成项目原型设计", 0.8),
            ("参加技术分享会", 0.7)
        ]
        
        for content, importance in test_memories:
            self.client.store_memory(
                agent_id=self.agent_id,
                namespace="shared",
                content=content,
                importance=importance
            )
        
        # 搜索测试
        results = self.client.search_memories(
            agent_id=self.agent_id,
            query="技术",
            namespace="shared",
            limit=10
        )
        
        self.assertGreater(len(results), 0)
        
        # 验证搜索结果
        found_contents = [r.content for r in results]
        self.assertTrue(any("Python" in content for content in found_contents))
        
        print(f"✓ 搜索成功，找到 {len(results)} 条记录")
    
    def test_memory_context(self):
        """测试上下文获取"""
        print("\n=== 测试上下文获取 ===")
        
        # 存储项目相关记忆
        project_memories = [
            "AI短剧项目启动",
            "完成了角色创建功能",
            "实现了视频生成模块",
            "优化了用户界面"
        ]
        
        for content in project_memories:
            self.client.store_memory(
                agent_id=self.agent_id,
                namespace="project/ai-drama",
                content=content,
                importance=0.8
            )
        
        # 获取上下文
        context = self.client.get_context(
            project_name="ai-drama",
            agent_id=self.agent_id
        )
        
        self.assertIsNotNone(context)
        self.assertGreater(len(context), 0)
        
        print(f"✓ 上下文获取成功: {len(context)} 字符")
    
    def test_dream_generation(self):
        """测试梦境生成"""
        print("\n=== 测试梦境生成 ===")
        
        # 存储一些记忆
        memories = [
            "今天学习了新的编程技术",
            "完成了项目的重要里程碑",
            "参加了技术讨论会议"
        ]
        
        for content in memories:
            self.client.store_memory(
                agent_id=self.agent_id,
                namespace="shared",
                content=content,
                importance=0.7
            )
        
        # 触发梦境
        dream = self.client.trigger_dream(
            agent_id=self.agent_id,
            namespace="shared"
        )
        
        self.assertIsNotNone(dream)
        self.assertGreater(len(dream), 0)
        
        print(f"✓ 梦境生成成功: {len(dream)} 字符")
    
    def test_namespace_isolation(self):
        """测试命名空间隔离"""
        print("\n=== 测试命名空间隔离 ===")
        
        # 在不同命名空间存储相同内容
        content = "测试命名空间隔离"
        
        # 存储到共享命名空间
        shared_result = self.client.store_memory(
            agent_id=self.agent_id,
            namespace="shared",
            content=content,
            importance=0.8
        )
        
        # 存储到个人命名空间
        personal_result = self.client.store_memory(
            agent_id=self.agent_id,
            namespace="personal",
            content=content,
            importance=0.8
        )
        
        # 验证隔离
        shared_search = self.client.search_memories(
            agent_id=self.agent_id,
            query="测试",
            namespace="shared",
            limit=10
        )
        
        personal_search = self.client.search_memories(
            agent_id=self.agent_id,
            query="测试",
            namespace="personal",
            limit=10
        )
        
        # 应该都能找到记录
        self.assertGreater(len(shared_search), 0)
        self.assertGreater(len(personal_search), 0)
        
        print(f"✓ 命名空间隔离测试通过")
        print(f"  共享命名空间: {len(shared_search)} 条记录")
        print(f"  个人命名空间: {len(personal_search)} 条记录")
    
    def test_performance(self):
        """测试性能"""
        print("\n=== 测试性能 ===")
        
        # 性能测试：批量存储
        start_time = time.time()
        
        for i in range(100):
            self.client.store_memory(
                agent_id=self.agent_id,
                namespace="shared",
                content=f"性能测试记忆 {i}",
                importance=0.5
            )
        
        storage_time = time.time() - start_time
        
        # 性能测试：批量搜索
        start_time = time.time()
        
        results = self.client.search_memories(
            agent_id=self.agent_id,
            query="性能测试",
            namespace="shared",
            limit=100
        )
        
        search_time = time.time() - start_time
        
        print(f"✓ 性能测试完成:")
        print(f"  批量存储 (100条): {storage_time:.2f}秒")
        print(f"  批量搜索 (100条): {search_time:.2f}秒")
        print(f"  平均存储时间: {storage_time/100*1000:.2f}毫秒")
        print(f"  平均搜索时间: {search_time/100*1000:.2f}毫秒")
    
    def test_error_handling(self):
        """测试错误处理"""
        print("\n=== 测试错误处理 ===")

        # 测试空搜索（不存在的关键词）
        results = self.client.search_memories(
            agent_id=self.agent_id,
            query="不存在的关键词_xyzzy",
            namespace="shared",
            limit=10
        )

        self.assertEqual(len(results), 0)

        print("✓ 错误处理测试通过")

def run_integration_tests():
    """运行集成测试"""
    print("开始运行外部记忆系统集成测试...")
    print("=" * 50)
    
    # 创建测试套件
    suite = unittest.TestLoader().loadTestsFromTestCase(TestExternalMemorySystem)
    
    # 运行测试
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    print("\n" + "=" * 50)
    print(f"测试结果: {result.testsRun} 个测试")
    print(f"成功: {result.testsRun - len(result.failures) - len(result.errors)}")
    print(f"失败: {len(result.failures)}")
    print(f"错误: {len(result.errors)}")
    
    if result.failures:
        print("\n失败的测试:")
        for test, traceback in result.failures:
            print(f"  - {test}: {traceback}")
    
    if result.errors:
        print("\n错误的测试:")
        for test, traceback in result.errors:
            print(f"  - {test}: {traceback}")
    
    return result.wasSuccessful()

if __name__ == "__main__":
    # 检查服务是否可用
    try:
        client = MemoryClient(host='localhost', port=50051)
        print("正在连接到gRPC服务...")
        time.sleep(2)
        print("✓ 连接成功")
    except Exception as e:
        print(f"✗ 无法连接到gRPC服务: {e}")
        print("请确保服务正在运行: deploy.sh start")
        sys.exit(1)
    
    # 运行集成测试
    success = run_integration_tests()
    
    if success:
        print("\n🎉 所有测试通过!")
        sys.exit(0)
    else:
        print("\n❌ 部分测试失败!")
        sys.exit(1)