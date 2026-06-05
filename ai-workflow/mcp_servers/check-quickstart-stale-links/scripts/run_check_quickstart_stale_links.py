#!/usr/bin/env python3
import sys
from pathlib import Path

# Add lib to path for common_utils
LIB_PATH = Path(__file__).resolve().parents[2] / "lib"
if str(LIB_PATH) not in sys.path:
    sys.path.insert(0, str(LIB_PATH))

from common_utils import inject_workflow_source, mcp_main

inject_workflow_source()
from workflow_kit.common.read_only_bundle import check_quickstart_stale_links_payload

def build_args(parser):
    parser.add_argument("--quickstart-path", action="append", dest="quickstart_paths", required=True)
    parser.add_argument("--project-profile-path")
    parser.add_argument("--session-handoff-path")
    parser.add_argument("--work-backlog-index-path")
    parser.add_argument("--agents-path")

def main():
    mcp_main(
        description="Run check_quickstart_stale_links MCP prototype.",
        arg_builder=build_args,
        payload_func=check_quickstart_stale_links_payload
    )

if __name__ == "__main__":
    main()
