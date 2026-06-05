#!/usr/bin/env python3
import sys
from pathlib import Path

# Add lib to path for common_utils
LIB_PATH = Path(__file__).resolve().parents[2] / "lib"
if str(LIB_PATH) not in sys.path:
    sys.path.insert(0, str(LIB_PATH))

from common_utils import inject_workflow_source, mcp_main

inject_workflow_source()
from workflow_kit.common.read_only_bundle import latest_backlog_payload

def build_args(parser):
    parser.add_argument("--backlog-dir-path")
    parser.add_argument("--work-backlog-index-path")

def main():
    mcp_main(
        description="Run latest_backlog MCP prototype.",
        arg_builder=build_args,
        payload_func=latest_backlog_payload
    )

if __name__ == "__main__":
    main()
