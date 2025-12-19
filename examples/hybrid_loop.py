#!/usr/bin/env python3
"""Hybrid Loop Example: Combining MossAPI and MCP tools.

This example demonstrates using CompositeToolExecutor to create loops
that mix local structural tools (MossAPI) with external MCP servers.

Use case: A code analysis loop that:
1. Uses MossAPI for structural analysis (skeleton, complexity)
2. Uses an external MCP server for additional capabilities

The CompositeToolExecutor routes tools based on prefix:
- "moss." prefix -> MossToolExecutor (local structural tools)
- "mcp." prefix -> MCPToolExecutor (external MCP server)
- "llm." prefix -> LLMToolExecutor (LLM calls)
"""

import asyncio
from pathlib import Path

from moss.agent_loop import (
    AgentLoop,
    AgentLoopRunner,
    CompositeToolExecutor,
    LLMConfig,
    LLMToolExecutor,
    LoopStep,
    MCPServerConfig,
    MCPToolExecutor,
    MossToolExecutor,
    StepType,
    dump_loop_yaml,
)


def create_hybrid_loop() -> AgentLoop:
    """Create a hybrid analysis loop.

    This loop demonstrates mixing different tool sources:
    - moss.skeleton: Local structural tool via MossAPI
    - moss.complexity: Local analysis tool via MossAPI
    - llm.analyze: LLM-based analysis

    In a full setup, you'd also include MCP tools like:
    - mcp.read_file: External filesystem access via MCP
    - mcp.search: External search capabilities
    """
    return AgentLoop(
        name="hybrid_analysis",
        steps=[
            # Step 1: Get structural overview via MossAPI
            LoopStep(
                name="skeleton",
                tool="moss.skeleton.format",
                step_type=StepType.TOOL,
            ),
            # Step 2: Analyze complexity via MossAPI
            LoopStep(
                name="complexity",
                tool="moss.complexity.analyze",
                step_type=StepType.TOOL,
                input_from="skeleton",
            ),
            # Step 3: LLM analysis combining both inputs
            LoopStep(
                name="analyze",
                tool="llm.analyze",
                step_type=StepType.LLM,
                input_from="complexity",
            ),
        ],
        entry="skeleton",
        exit_conditions=["analyze.complete"],
        max_steps=5,
    )


def create_composite_executor(
    root: Path | None = None,
    mcp_config: MCPServerConfig | None = None,
    llm_config: LLMConfig | None = None,
) -> CompositeToolExecutor:
    """Create a composite executor with multiple backends.

    Args:
        root: Project root for MossAPI tools
        mcp_config: Optional MCP server configuration
        llm_config: Optional LLM configuration

    Returns:
        CompositeToolExecutor that routes tools by prefix
    """
    # MossAPI executor for structural tools
    moss_executor = MossToolExecutor(root=root)

    # LLM executor for generation/analysis
    llm_executor = LLMToolExecutor(
        config=llm_config or LLMConfig(mock=True),
        moss_executor=moss_executor,
        root=root,
    )

    # Build executor map
    executors: dict[str, any] = {
        "moss.": moss_executor,
        "llm.": llm_executor,
    }

    # Optionally add MCP executor if config provided
    # Note: MCP executor requires async connection, handled separately
    if mcp_config:
        mcp_executor = MCPToolExecutor(mcp_config)
        executors["mcp."] = mcp_executor

    return CompositeToolExecutor(
        executors=executors,
        default=moss_executor,  # Fallback for unprefixed tools
    )


async def run_hybrid_example():
    """Run the hybrid loop example."""
    print("=== Hybrid Loop Example ===\n")

    # Create the loop
    loop = create_hybrid_loop()

    # Show the loop definition
    print("Loop definition (YAML):")
    print("-" * 40)
    print(dump_loop_yaml(loop))
    print("-" * 40)

    # Create executor (mock mode for demo)
    executor = create_composite_executor(
        root=Path("."),
        llm_config=LLMConfig(mock=True),
    )

    # Run the loop
    print("\nRunning hybrid loop...")
    runner = AgentLoopRunner(executor)

    # Provide initial context (file to analyze)
    initial_input = {"file_path": "src/moss/agent_loop.py"}

    result = await runner.run(loop, initial_input)

    # Print results
    print(f"\nLoop completed: {result.status.name}")
    print(f"Steps executed: {len(result.step_results)}")
    print(f"Final status: {'SUCCESS' if result.success else 'FAILED'}")

    # Print metrics
    print("\nMetrics:")
    print(f"  LLM calls: {result.metrics.llm_calls}")
    print(f"  Tool calls: {result.metrics.tool_calls}")
    print(f"  Total time: {result.metrics.wall_time_seconds:.2f}s")

    return result


async def run_with_mcp_example():
    """Example of running with an actual MCP server.

    This example shows how to set up a loop with external MCP tools.
    Requires an MCP server to be available (e.g., moss's own MCP server).
    """
    print("=== Hybrid Loop with MCP Server ===\n")

    # MCP server config (e.g., moss's own MCP server)
    mcp_config = MCPServerConfig(
        command="uv",
        args=["run", "python", "-m", "moss.mcp_server"],
        cwd=str(Path(__file__).parent.parent),
    )

    # Create a loop that uses both moss tools and MCP tools
    loop = AgentLoop(
        name="mcp_hybrid",
        steps=[
            # Use moss MCP server's skeleton tool
            LoopStep(
                name="mcp_skeleton",
                tool="mcp.skeleton_format",
                step_type=StepType.TOOL,
            ),
            # Use local LLM for analysis
            LoopStep(
                name="analyze",
                tool="llm.summarize",
                step_type=StepType.LLM,
                input_from="mcp_skeleton",
            ),
        ],
        entry="mcp_skeleton",
        exit_conditions=["analyze.complete"],
        max_steps=3,
    )

    # Create executor with MCP
    mcp_executor = MCPToolExecutor(mcp_config)
    moss_executor = MossToolExecutor()
    llm_executor = LLMToolExecutor(
        config=LLMConfig(mock=True),
        moss_executor=moss_executor,
    )

    composite = CompositeToolExecutor(
        executors={
            "mcp.": mcp_executor,
            "moss.": moss_executor,
            "llm.": llm_executor,
        }
    )

    # Connect to MCP server
    print("Connecting to MCP server...")
    try:
        await mcp_executor.connect()
        print(f"Connected! Available tools: {len(mcp_executor.list_tools())}")

        # Run the loop
        runner = AgentLoopRunner(composite)
        result = await runner.run(loop, {"file_path": "src/moss/agent_loop.py"})

        print(f"\nLoop completed: {result.status.name}")

    except Exception as e:
        print(f"MCP connection failed: {e}")
        print("(This is expected if no MCP server is running)")

    finally:
        await mcp_executor.disconnect()


def main():
    """Main entry point."""
    print("Hybrid Loop Examples")
    print("=" * 50)
    print()

    # Run basic hybrid example (no MCP server needed)
    asyncio.run(run_hybrid_example())

    print("\n" + "=" * 50)
    print()

    # Optionally run MCP example (requires server)
    # Uncomment to test with actual MCP server:
    # asyncio.run(run_with_mcp_example())


if __name__ == "__main__":
    main()
