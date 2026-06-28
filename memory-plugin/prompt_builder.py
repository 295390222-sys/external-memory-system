import logging

logger = logging.getLogger(__name__)


class PromptBuilder:
    def __init__(self, max_tokens: int = 8192):
        self.max_tokens = max_tokens

    def build(
        self,
        user_query: str,
        shared_memories: list = None,
        project_memories: list = None,
        agent_memories: list = None,
        working_memory: list = None,
    ) -> str:
        sections = []
        budget = self.max_tokens

        # 1. Working Memory (临时, 最短)
        if working_memory:
            text = self._format_section("Working Memory", working_memory, budget // 8)
            if text:
                sections.append(text)
                budget -= len(text) // 2

        # 2. Shared Memory (全局共享)
        if shared_memories:
            text = self._format_section("Shared Memory", shared_memories, budget // 3)
            if text:
                sections.append(text)
                budget -= len(text) // 2

        # 3. Agent Memory (当前Agent)
        if agent_memories:
            text = self._format_section("Agent Memory", agent_memories, budget // 3)
            if text:
                sections.append(text)
                budget -= len(text) // 2

        # 4. Project Memory (当前项目)
        if project_memories:
            text = self._format_section("Project Context", project_memories, budget // 3)
            if text:
                sections.append(text)

        # 5. User Query
        sections.append(f"User: {user_query}")

        return "\n\n".join(sections)

    def _format_section(self, title: str, items: list, token_budget: int) -> str | None:
        if not items:
            return None

        lines = [f"=== {title} ==="]
        used = len(lines[0]) // 2

        for item in items:
            content = item if isinstance(item, str) else getattr(item, "content", str(item))
            if hasattr(item, "importance"):
                prefix = f"[{item.importance}] "
            else:
                prefix = ""

            if hasattr(item, "namespace") and item.namespace:
                prefix = f"[{item.namespace}] {prefix}"

            entry = f"{prefix}{content[:300]}"
            est = len(entry) // 2

            if used + est > token_budget:
                break

            lines.append(entry)
            used += est

        return "\n".join(lines)

    def build_system_prompt(self, agent_name: str, capabilities: list[str] = None) -> str:
        parts = [f"You are {agent_name}."]
        if capabilities:
            parts.append(f"Capabilities: {', '.join(capabilities)}.")
        parts.append("Use the provided memory context to inform your responses.")
        return " ".join(parts)
