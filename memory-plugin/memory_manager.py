import time
import logging
from grpc_client import MemoryGrpcClient, MemoryRecord

logger = logging.getLogger(__name__)


class MemoryManager:
    def __init__(self, grpc: MemoryGrpcClient, config: dict):
        self.grpc = grpc
        self.scoring_config = config["memory"]["scoring"]

    async def save(self, agent_id: str, content: str, namespace: str = "shared"):
        importance = self._score(content)
        now = int(time.time() * 1000)

        record = MemoryRecord(
            agent_id=agent_id,
            namespace=namespace,
            importance=importance,
            content=content,
            created_at=now,
            updated_at=now,
            access_count=0,
            last_access=now,
        )

        if importance < self.scoring_config["threshold_working"]:
            record.memory_type = 0
            record.expire_at = now + 3600000
        elif importance < self.scoring_config["threshold_episodic"]:
            record.memory_type = 1
            record.expire_at = now + 86400000
        else:
            record.memory_type = 2
            record.expire_at = None

        await self.grpc.store(record)

    def _score(self, content: str) -> int:
        score = 5

        if len(content) > 500:
            score += 1
        if len(content) > 2000:
            score += 1

        important_keywords = [
            "project", "deploy", "release", "production", "critical",
            "bug", "fix", "migration", "api", "database",
            "architecture", "decision", "goal", "milestone",
            "完成", "修复", "部署", "上线", "架构",
        ]
        lower = content.lower()
        if any(kw in lower for kw in important_keywords):
            score += 1

        code_markers = ["{", "}", ";", "fn ", "pub ", "struct", "impl", "async"]
        if any(m in content for m in code_markers):
            score += 1

        return max(0, min(10, score))
