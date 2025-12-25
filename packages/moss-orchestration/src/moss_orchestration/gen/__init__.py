"""Interface generators for MossAPI.

This module provides automatic interface generation from the MossAPI:
- CLI: Generate argparse commands from API methods
- Compact: Token-efficient tool signatures for LLM consumption (93% smaller than MCP)
- gRPC: Generate Protocol Buffers and servicer implementations
- HTTP: Generate FastAPI routes from API methods
- LSP: Generate workspace commands for Language Server Protocol
- MCP: Generate Model Context Protocol tools from API methods
- OpenAPI: Generate OpenAPI specification from API
"""

from moss_orchestration.gen.base import LazyAPIExecutor
from moss_orchestration.gen.cli import CLIGenerator, generate_cli
from moss_orchestration.gen.compact import (
    generate_compact_by_category,
    generate_compact_tools,
)
from moss_orchestration.gen.grpc import GRPCGenerator, generate_proto, generate_servicer_code
from moss_orchestration.gen.http import HTTPGenerator, generate_http, generate_openapi
from moss_orchestration.gen.introspect import (
    APIMethod,
    APIParameter,
    SubAPI,
    introspect_api,
)
from moss_orchestration.gen.lsp import LSPGenerator, generate_lsp_commands, get_command_list
from moss_orchestration.gen.mcp import MCPGenerator, generate_mcp, generate_mcp_definitions

__all__ = [
    "APIMethod",
    "APIParameter",
    "CLIGenerator",
    "GRPCGenerator",
    "HTTPGenerator",
    "LSPGenerator",
    "LazyAPIExecutor",
    "MCPGenerator",
    "SubAPI",
    "generate_cli",
    "generate_compact_by_category",
    "generate_compact_tools",
    "generate_http",
    "generate_lsp_commands",
    "generate_mcp",
    "generate_mcp_definitions",
    "generate_openapi",
    "generate_proto",
    "generate_servicer_code",
    "get_command_list",
    "introspect_api",
]
