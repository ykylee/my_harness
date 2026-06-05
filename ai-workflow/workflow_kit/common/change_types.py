"""Changed-file classification helpers shared across workflow kit prototypes."""

from __future__ import annotations


def classify_doc_sync_file(path_str: str) -> str:
    lower = path_str.lower()
    if lower.endswith(".md"):
        if "readme" in lower:
            return "hub_doc"
        if "runbook" in lower:
            return "runbook_doc"
        if "handoff" in lower:
            return "handoff_doc"
        if "backlog" in lower:
            return "backlog_doc"
        return "doc"
    if any(lower.endswith(ext) for ext in [".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".rs", ".java"]):
        return "code"
    if any(lower.endswith(ext) for ext in [".yaml", ".yml", ".json", ".toml", ".ini"]):
        return "config"
    return "other"


def classify_impacted_doc_file(path_str: str) -> str:
    lower = path_str.lower()
    if lower.endswith(".md"):
        return "doc"
    if any(lower.endswith(ext) for ext in [".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".rs", ".java"]):
        return "code"
    if any(lower.endswith(ext) for ext in [".yaml", ".yml", ".json", ".toml", ".ini"]):
        return "config"
    return "other"


def classify_validation_change_kinds(path_str: str) -> set[str]:
    lower = path_str.lower()
    kinds: set[str] = set()

    if any(token in lower for token in ["frontend/", "/ui/", "web/", ".tsx", ".jsx", ".css", ".scss"]):
        kinds.add("ui")
    if any(token in lower for token in ["deploy", "helm", "k8s", "infra", "terraform", "ops", "runbook"]):
        kinds.add("ops")
    if any(token in lower for token in ["prompt", "eval", "dataset", "manifest", "report"]):
        kinds.add("prompt_or_eval")
    if lower.endswith(".md"):
        kinds.add("docs")
    if any(lower.endswith(ext) for ext in [".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".rs", ".java"]):
        kinds.add("code")
    if any(lower.endswith(ext) for ext in [".yaml", ".yml", ".json", ".toml", ".ini"]):
        kinds.add("config")

    if not kinds:
        kinds.add("other")
    return kinds


def classify_index_change_kinds(path_str: str) -> set[str]:
    lower = path_str.lower()
    kinds: set[str] = set()
    is_markdown = lower.endswith(".md")

    if lower.endswith("readme.md"):
        kinds.add("root_or_hub_readme")
    if is_markdown:
        kinds.add("doc")
    if is_markdown and any(
        token in lower
        for token in ["runbook", "/reports/", "release-report", "/dataset", "manifest", "/prompt", "/prompts/"]
    ):
        kinds.add("hub_child_doc")
    if "backlog" in lower:
        kinds.add("backlog_doc")
    if "handoff" in lower:
        kinds.add("handoff_doc")
    if any(lower.endswith(ext) for ext in [".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".rs", ".java"]):
        kinds.add("code")
    if any(lower.endswith(ext) for ext in [".yaml", ".yml", ".json", ".toml", ".ini"]):
        kinds.add("config")
    if lower.startswith("docs/") and is_markdown and lower.count("/") >= 2:
        kinds.add("nested_doc")

    if not kinds:
        kinds.add("other")
    return kinds


def detect_validation_change_types(changed_files: list[str], change_summary: str | None) -> list[str]:
    detected: list[str] = []
    for item in changed_files:
        detected.extend(sorted(classify_validation_change_kinds(item)))

    summary_lower = (change_summary or "").lower()
    if any(token in summary_lower for token in ["ui", "screen", "console", "frontend"]):
        detected.append("ui")
    if any(token in summary_lower for token in ["deploy", "release", "runbook", "rollback", "healthcheck"]):
        detected.append("ops")
    if any(token in summary_lower for token in ["prompt", "eval", "dataset", "report", "experiment"]):
        detected.append("prompt_or_eval")
    if any(token in summary_lower for token in ["docs", "document", "문서", "runbook", "handoff", "backlog"]):
        detected.append("docs")
    if any(token in summary_lower for token in ["config", "setting", "schema", "migration"]):
        detected.append("config")
    if any(token in summary_lower for token in ["code", "api", "logic", "service", "handler"]):
        detected.append("code")

    return dedupe(detected or ["other"])


def dedupe(items: list[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for item in items:
        normalized = item.strip()
        if normalized and normalized not in seen:
            seen.add(normalized)
            result.append(normalized)
    return result
