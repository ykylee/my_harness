import re
from pathlib import Path

class DocTransformer:
    """Transforms human-readable documentation into AI-optimized versions."""

    @staticmethod
    def minify_markdown(content: str) -> str:
        """
        Minifies markdown by:
        1. Preserving headers, lists, and code blocks.
        2. Removing verbose prose/paragraphs.
        3. Compacting whitespace.
        """
        lines = content.splitlines()
        optimized_lines = []
        in_code_block = False

        for line in lines:
            stripped = line.strip()

            # Code blocks must be preserved entirely
            if stripped.startswith("```"):
                in_code_block = not in_code_block
                optimized_lines.append(line)
                continue

            if in_code_block:
                optimized_lines.append(line)
                continue

            # Preserve headers and list items
            if stripped.startswith("#") or stripped.startswith("-") or stripped.startswith("*") or re.match(r"^\d+\.", stripped):
                optimized_lines.append(line)
                continue

            # Preserve tables
            if "|" in stripped:
                optimized_lines.append(line)
                continue

            # Preserve metadata/frontmatter
            if stripped.startswith("---") or (len(optimized_lines) > 0 and optimized_lines[0].startswith("---") and not any(l.startswith("---") for l in optimized_lines[1:])):
                 optimized_lines.append(line)
                 continue

            # Remove plain prose unless it's a short instruction or contains keywords
            keywords = ["MUST", "SHOULD", "REQUIRED", "IMPORTANT", "WARNING", "NOTE", "원칙", "규칙"]
            if any(k in stripped.upper() for k in keywords) and len(stripped) < 200:
                optimized_lines.append(line)
                continue

            # Otherwise, skip prose to save tokens
            continue

        # Final compaction
        result = "\n".join(optimized_lines)
        # Remove multiple newlines
        result = re.sub(r"\n{3,}", "\n\n", result)
        return result.strip()

    def transform_file(self, source: Path, destination: Path):
        """Read source, minify, and write to destination."""
        content = source.read_text(encoding="utf-8")
        if source.suffix == ".md":
            optimized = self.minify_markdown(content)
        else:
            optimized = content # Keep JSON/other as is for now

        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(optimized, encoding="utf-8")
