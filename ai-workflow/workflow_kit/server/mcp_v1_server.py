"""Official MCP v1.0 SDK server wrapper."""

from __future__ import annotations

import sys
from typing import Any, Callable

try:
    from mcp.server.fastmcp import FastMCP
    HAS_FASTMCP = True
except ImportError:
    HAS_FASTMCP = False


class WorkflowMCPv1Server:
    """Wrapper for official MCP v1.0 server implementation."""

    def __init__(self, name: str, version: str = "1.0.0"):
        if not HAS_FASTMCP:
            print("Error: 'mcp' SDK not installed. Run 'pip install mcp' to use MCP v1.0 features.", file=sys.stderr)
            sys.exit(1)
        
        self.mcp = FastMCP(name)

    def tool(self, name: str | None = None, description: str | None = None) -> Callable:
        """Decorator to register a tool."""
        return self.mcp.tool(name=name, description=description)

    def run(self):
        """Run the server over stdio."""
        self.mcp.run()


def create_v1_server(name: str, version: str) -> WorkflowMCPv1Server:
    """Factory function for MCP v1.0 servers."""
    return WorkflowMCPv1Server(name, version=version)
