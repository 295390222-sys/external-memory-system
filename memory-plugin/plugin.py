import yaml
import logging
from pathlib import Path

from grpc_client import MemoryGrpcClient
from injector import ContextInjector
from memory_manager import MemoryManager

logger = logging.getLogger(__name__)


class MemoryPlugin:
    def __init__(self, config_path: str = None):
        self.config = self._load_config(config_path)
        self.grpc = MemoryGrpcClient(
            host=self.config["memory"]["grpc_host"],
            port=self.config["memory"]["grpc_port"],
        )
        self.injector = ContextInjector(
            max_tokens=self.config["memory"]["inject"]["max_context_tokens"],
            min_relevance=self.config["memory"]["inject"]["min_relevance"],
        )
        self.manager = MemoryManager(self.grpc, self.config)

    def _load_config(self, path: str | None) -> dict:
        if path is None:
            path = str(Path(__file__).parent / "config.yaml")
        with open(path) as f:
            return yaml.safe_load(f)

    async def on_message(self, agent_id: str, user_message: str, namespace: str = "shared"):
        logger.info(f"on_message: agent={agent_id}, ns={namespace}")

        search_results = await self.grpc.search(
            agent_id=agent_id,
            query=user_message,
            namespace=namespace,
            limit=10,
            use_keyword=True,
            use_vector=True,
            use_graph=True,
        )

        augmented = self.injector.inject(user_message, search_results)
        return augmented, search_results

    async def on_response(self, agent_id: str, response_text: str, namespace: str = "shared"):
        await self.manager.save(agent_id, response_text, namespace)
        logger.info(f"stored memory for agent={agent_id}")

    async def dream(self, agent_id: str, namespace: str = "shared"):
        result = await self.grpc.dream(agent_id=agent_id, namespace=namespace)
        logger.info(
            f"dream completed: {len(result.new_memories)} new memories, "
            f"{len(result.inferences)} inferences, "
            f"{len(result.hypotheses)} hypotheses"
        )
        return result

    async def get_context(self, project: str, agent_id: str):
        return await self.grpc.get_project_context(project=project, agent_id=agent_id)
