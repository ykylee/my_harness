"""Registry and logic for task-specific work modes."""

from __future__ import annotations

# Mode guidelines dictionary
MODE_GUIDELINES: dict[str, list[str]] = {
    "Analysis": [
        "가이드: 구조 분석 및 탐색 중심. `doc-worker`를 활용하여 병렬 탐색하고 `code_index`를 갱신한다.",
    ],
    "Requirements": [
        "가이드: 니즈 수집 및 명세화 중심. 사용자 질문을 통해 제약 사항을 명확히 하고 `requirements.md`를 작성한다.",
    ],
    "Design": [
        "가이드: 아키텍처 및 상세 설계 중심. `main`급 모델을 사용하여 `technical_spec.md`를 설계한다.",
    ],
    "Planning": [
        "가이드: 태스크 분해 및 일정 계획 중심. 목표를 최소 단위 작업으로 쪼개어 백로그에 등록한다.",
    ],
    "Implementation": [
        "가이드: 코드 작성 및 단위 검증 중심. `code-worker`와 `validation-worker` 루프를 통해 즉시 검증한다.",
    ],
    "Refactoring": [
        "가이드: 코드 개선 및 회귀 테스트 중심. 기능 변경 없이 품질을 높이고 `validation-worker`로 무결성을 확인한다.",
    ],
}

# Mode recommendation keywords
MODE_KEYWORDS: dict[str, list[str]] = {
    "Refactoring": ["refactor", "리팩터", "리팩토링", "개선", "cleanup", "정리"],
    "Implementation": ["implement", "구현", "feat", "add", "fix", "버그", "수정", "작업", "도입"],
    "Design": ["design", "설계", "architecture", "아키텍처"],
    "Planning": ["plan", "계획", "roadmap", "일정"],
    "Requirements": ["requirement", "요구", "명세", "needs"],
    "Analysis": ["analysis", "분석", "조사", "탐색", "investigate", "검토"],
}

def get_mode_guidelines(mode: str) -> list[str]:
    """Return guidelines for a specific mode."""
    return MODE_GUIDELINES.get(mode, [])

def recommend_mode_from_text(text: str) -> str | None:
    """Recommend a task mode based on input text."""
    t = text.lower()
    # Check priorities based on the order in MODE_KEYWORDS
    for mode, keywords in MODE_KEYWORDS.items():
        if any(k in t for k in keywords):
            return mode
    return None
