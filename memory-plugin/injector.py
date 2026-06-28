import logging

logger = logging.getLogger(__name__)


class ContextInjector:
    def __init__(self, max_tokens: int = 4096, min_relevance: float = 0.6):
        self.max_tokens = max_tokens
        self.min_relevance = min_relevance

    def inject(self, user_message: str, search_results: list) -> str:
        if not search_results:
            return user_message

        relevant = [
            r for r in search_results
            if r.importance >= 5
        ]

        if not relevant:
            return user_message

        context_parts = []
        total_est = 0

        for r in relevant:
            preview = r.content[:300]
            entry = f"[Memory:{r.importance}] {preview}"
            est_tokens = len(entry) // 2
            if total_est + est_tokens > self.max_tokens:
                break
            context_parts.append(entry)
            total_est += est_tokens

        if not context_parts:
            return user_message

        context_block = "\n".join(context_parts)
        return f"Relevant memories:\n{context_block}\n\nUser: {user_message}"

    def build_system_prompt(self, agent_name: str, memories: list) -> str:
        if not memories:
            return ""

        procedural = [m for m in memories if m.memory_type == 3]
        semantic = [m for m in memories if m.memory_type == 2]

        parts = []
        if procedural:
            parts.append("Procedural Knowledge:")
            for m in procedural[:5]:
                parts.append(f"  - {m.content[:200]}")

        if semantic:
            parts.append("Semantic Knowledge:")
            for m in semantic[:5]:
                parts.append(f"  - {m.content[:200]}")

        return "\n".join(parts)
