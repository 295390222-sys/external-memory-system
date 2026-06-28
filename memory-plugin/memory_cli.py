#!/Users/wangjuncong/memory-system/venv/bin/python3
"""
CLI interface for memory-system.
OpenClaw agent can call this via shell commands.

Usage:
    memory-cli store <agent_id> <namespace> <text>
    memory-cli search <agent_id> <query> [--limit 10] [--ns shared]
    memory-cli dream <agent_id> [--ns shared]
    memory-cli context <project> <agent_id>
    memory-clic summary <agent_id> [--ns shared]
"""
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import json
import asyncio
import argparse
from grpc_client import MemoryGrpcClient, MemoryRecord


async def cmd_store(args):
    client = MemoryGrpcClient()
    now = __import__("time").time()
    record = MemoryRecord(
        agent_id=args.agent_id,
        namespace=args.namespace,
        importance=5,
        content=args.text,
        memory_type=2,
        created_at=int(now * 1000),
        updated_at=int(now * 1000),
        access_count=0,
        last_access=int(now * 1000),
    )
    id = await client.store(record)
    print(json.dumps({"id": id, "success": True}))


async def cmd_search(args):
    client = MemoryGrpcClient()
    results = await client.search(
        agent_id=args.agent_id,
        query=args.query,
        namespace=args.namespace,
        limit=args.limit,
        use_keyword=True,
        use_vector=True,
    )
    output = []
    for r in results:
        output.append({
            "id": r.id,
            "importance": r.importance,
            "content": r.content[:200],
            "namespace": r.namespace,
            "created_at": r.created_at,
        })
    print(json.dumps(output, ensure_ascii=False))


async def cmd_dream(args):
    client = MemoryGrpcClient()
    result = await client.dream(agent_id=args.agent_id, namespace=args.namespace)
    print(json.dumps({
        "facts": len(result.new_memories),
        "inferences": result.inferences,
        "hypotheses": result.hypotheses,
    }, ensure_ascii=False))


async def cmd_context(args):
    client = MemoryGrpcClient()
    ctx = await client.get_project_context(project=args.project, agent_id=args.agent_id)
    print(ctx.context)


async def cmd_summary(args):
    client = MemoryGrpcClient()
    summary, points, mtype = await client.summarize(
        agent_id=args.agent_id,
        namespace=args.namespace,
    )
    print(json.dumps({
        "summary": summary,
        "key_points": points,
        "type": mtype,
    }, ensure_ascii=False))


def main():
    parser = argparse.ArgumentParser(description="Memory System CLI")
    sub = parser.add_subparsers(dest="command")

    p_store = sub.add_parser("store")
    p_store.add_argument("agent_id")
    p_store.add_argument("namespace")
    p_store.add_argument("text")

    p_search = sub.add_parser("search")
    p_search.add_argument("agent_id")
    p_search.add_argument("query")
    p_search.add_argument("--limit", type=int, default=10)
    p_search.add_argument("--ns", dest="namespace", default="shared")

    p_dream = sub.add_parser("dream")
    p_dream.add_argument("agent_id")
    p_dream.add_argument("--ns", dest="namespace", default="shared")

    p_context = sub.add_parser("context")
    p_context.add_argument("project")
    p_context.add_argument("agent_id")

    p_summary = sub.add_parser("summary")
    p_summary.add_argument("agent_id")
    p_summary.add_argument("--ns", dest="namespace", default="shared")

    args = parser.parse_args()
    if not args.command:
        parser.print_help()
        sys.exit(1)

    asyncio.run(globals()[f"cmd_{args.command}"](args))


if __name__ == "__main__":
    main()
