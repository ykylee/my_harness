#!/usr/bin/env python3
"""Bootstrap a reusable standard AI workflow kit into a target repository.

**This module is a thin compatibility shim.** The implementation lives in
the :mod:`bootstrap_lib` package; this file just re-exports the public
API and forwards the CLI entry point so that legacy invocations
(``python bootstrap_workflow_kit.py --help``, etc.) keep working.

For new code, prefer:

- ``python -m bootstrap_lib`` — CLI entry point
- ``from bootstrap_lib import ...`` — programmatic API
"""

from __future__ import annotations

import sys
from pathlib import Path

# Make sure ``bootstrap_lib`` is importable when this file is executed directly
# via ``python bootstrap_workflow_kit.py``.
_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

# Re-export every public symbol from the bootstrap_lib package so that
# ``from bootstrap_workflow_kit import …`` keeps working for any downstream
# caller that still imports the legacy module name.
from bootstrap_lib.__main__ import (  # noqa: E402,F401
    DEFAULT_CORE_DOCS,
    DEFAULT_CORE_SUPPORT_PATHS,
    HARNESS_DEFINITIONS,
    build_manifest,
    enforce_harness_selection,
    infer_project_context,
    main,
    make_paths,
    parse_args,
    prompt_for_harnesses,
    selected_harnesses,
    update_dependencies,
    write_harness_files,
)
from bootstrap_lib.harnesses import (  # noqa: E402,F401
    HARNESS_FILE_BUILDERS,
    HARNESS_SPECS,
    SUPPORTED_HARNESSES,
    HarnessSpec,
    spec_for,
)
from bootstrap_lib.paths import HarnessDefinition, Paths  # noqa: E402,F401

# CLI entry point — preserves the original ``python bootstrap_workflow_kit.py`` UX.
if __name__ == "__main__":
    raise SystemExit(main())
