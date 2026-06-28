import grpc
from typing import Optional
from dataclasses import dataclass, field
from google.protobuf.json_format import MessageToDict


@dataclass
class MemoryRecord:
    id: str = ""
    agent_id: str = ""
    namespace: str = "shared"
    memory_type: int = 0
    importance: int = 0
    content: str = ""
    embedding: list[float] = field(default_factory=list)
    entities: list[dict] = field(default_factory=list)
    created_at: int = 0
    updated_at: int = 0
    access_count: int = 0
    last_access: int = 0
    expire_at: Optional[int] = None


@dataclass
class DreamResult:
    new_memories: list[MemoryRecord] = field(default_factory=list)
    patterns: list[str] = field(default_factory=list)
    inferences: list[str] = field(default_factory=list)
    hypotheses: list[str] = field(default_factory=list)


@dataclass
class ProjectContext:
    context: str = ""
    recent_memories: list[MemoryRecord] = field(default_factory=list)
    related_projects: list[str] = field(default_factory=list)


class MemoryGrpcClient:
    def __init__(self, host: str = "127.0.0.1", port: int = 50051):
        self.address = f"{host}:{port}"
        self._channel: Optional[grpc.aio.Channel] = None

    async def _get_stub(self):
        from memory_pb2_grpc import MemoryServiceStub
        if self._channel is None:
            self._channel = grpc.aio.insecure_channel(self.address)
        return MemoryServiceStub(self._channel)

    async def store(self, record: MemoryRecord) -> str:
        from memory_pb2 import StoreRequest, MemoryRecord as ProtoRecord

        stub = await self._get_stub()
        proto = ProtoRecord(
            id=record.id,
            agent_id=record.agent_id,
            namespace=record.namespace,
            memory_type=record.memory_type,
            importance=record.importance,
            content=record.content,
            created_at=record.created_at,
            updated_at=record.updated_at,
            access_count=record.access_count,
            last_access=record.last_access,
        )
        if record.expire_at:
            proto.expire_at = record.expire_at

        resp = await stub.StoreMemory(StoreRequest(record=proto))
        return resp.id

    async def search(
        self,
        agent_id: str,
        query: str,
        namespace: str = "shared",
        limit: int = 10,
        memory_type: Optional[int] = None,
        use_keyword: bool = True,
        use_vector: bool = True,
        use_graph: bool = False,
    ) -> list[MemoryRecord]:
        from memory_pb2 import SearchRequest

        stub = await self._get_stub()
        req = SearchRequest(
            agent_id=agent_id,
            query=query,
            namespace=namespace,
            limit=limit,
            use_keyword=use_keyword,
            use_vector=use_vector,
            use_graph=use_graph,
        )
        if memory_type is not None:
            req.memory_type = memory_type

        resp = await stub.SearchMemory(req)
        return [_proto_to_record(r) for r in resp.results]

    async def delete(self, id: str, agent_id: str, namespace: str = "shared") -> bool:
        from memory_pb2 import DeleteRequest

        stub = await self._get_stub()
        resp = await stub.DeleteMemory(DeleteRequest(id=id, agent_id=agent_id, namespace=namespace))
        return resp.success

    async def summarize(
        self,
        agent_id: str,
        namespace: str = "shared",
        since: int = 0,
        until: Optional[int] = None,
    ) -> tuple[str, list[str], str]:
        from memory_pb2 import SummaryRequest

        stub = await self._get_stub()
        import time
        req = SummaryRequest(
            agent_id=agent_id,
            namespace=namespace,
            since=since,
            until=until or int(time.time() * 1000),
        )
        resp = await stub.SummarizeMemory(req)
        return resp.summary, list(resp.key_points), resp.memory_type

    async def dream(self, agent_id: str, namespace: str = "shared") -> DreamResult:
        from memory_pb2 import DreamRequest

        stub = await self._get_stub()
        resp = await stub.Dreaming(DreamRequest(agent_id=agent_id, namespace=namespace))
        return DreamResult(
            new_memories=[_proto_to_record(r) for r in resp.new_memories],
            patterns=list(resp.patterns),
            inferences=list(resp.inferences),
            hypotheses=list(resp.hypotheses),
        )

    async def get_project_context(self, project: str, agent_id: str) -> ProjectContext:
        from memory_pb2 import ProjectRequest

        stub = await self._get_stub()
        resp = await stub.GetProjectContext(ProjectRequest(project=project, agent_id=agent_id))
        return ProjectContext(
            context=resp.context,
            recent_memories=[_proto_to_record(r) for r in resp.recent_memories],
            related_projects=list(resp.related_projects),
        )

    async def close(self):
        if self._channel:
            await self._channel.close()
            self._channel = None


class MemoryClient:
    """同步适配器，供集成测试和同步代码使用。"""

    def __init__(self, host: str = "127.0.0.1", port: int = 50051):
        self._async = MemoryGrpcClient(host, port)

    def _run_async(self, coro):
        import asyncio
        async def run_and_close():
            try:
                return await coro
            finally:
                await self._async.close()
        return asyncio.run(run_and_close())

    def store_memory(self, agent_id: str, namespace: str, content: str, importance: int = 5) -> MemoryRecord:
        record = MemoryRecord(agent_id=agent_id, namespace=namespace, content=content, importance=importance)
        record.id = self._run_async(self._async.store(record))
        return record

    def search_memories(self, agent_id: str, query: str, namespace: str = "shared", limit: int = 10) -> list:
        return self._run_async(self._async.search(agent_id, query, namespace, limit))

    def get_context(self, project_name: str, agent_id: str) -> str:
        ctx = self._run_async(self._async.get_project_context(project_name, agent_id))
        return ctx.context

    def trigger_dream(self, agent_id: str, namespace: str = "shared") -> str:
        dream = self._run_async(self._async.dream(agent_id, namespace))
        return "\n".join(dream.inferences) if dream.inferences else "梦境生成完成"

    def clear_memories(self, agent_id: str):
        pass


def _proto_to_record(proto) -> MemoryRecord:
    return MemoryRecord(
        id=proto.id,
        agent_id=proto.agent_id,
        namespace=proto.namespace,
        memory_type=proto.memory_type,
        importance=proto.importance,
        content=proto.content,
        embedding=list(proto.embedding),
        entities=[{"name": e.name, "relation": e.relation} for e in proto.entities],
        created_at=proto.created_at,
        updated_at=proto.updated_at,
        access_count=proto.access_count,
        last_access=proto.last_access,
        expire_at=proto.expire_at if proto.HasField("expire_at") else None,
    )
