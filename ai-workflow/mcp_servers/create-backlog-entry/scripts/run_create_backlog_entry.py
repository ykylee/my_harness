#!/usr/bin/env python3
import sys
from pathlib import Path

# Add lib to path for common_utils
LIB_PATH = Path(__file__).resolve().parents[2] / "lib"
if str(LIB_PATH) not in sys.path:
    sys.path.insert(0, str(LIB_PATH))

from common_utils import inject_workflow_source, mcp_main

inject_workflow_source()
from workflow_kit.common.read_only_bundle import create_backlog_entry_payload

def build_args(parser):
    parser.add_argument("--task-id", required=True)
    parser.add_argument("--task-name", required=True)
    parser.add_argument("--request-date", required=True)
    parser.add_argument("--status", default="planned")
    parser.add_argument("--priority", default="high")

def main():
    mcp_main(
        description="Run create_backlog_entry MCP prototype.",
        arg_builder=build_args,
        payload_func=create_backlog_entry_payload
    )

if __name__ == "__main__":
    main()
