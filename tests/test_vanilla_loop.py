from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from moss.agent_loop import LLMConfig, ToolExecutor
from moss.moss_api import MossAPI
from moss.vanilla_loop import VanillaAgentLoop, VanillaLoopState


@pytest.fixture
def mock_api():
    api = MagicMock(spec=MossAPI)
    api.root = "/project"
    return api


@pytest.fixture
def mock_executor():
    return AsyncMock(spec=ToolExecutor)


@pytest.fixture
def llm_config():
    return LLMConfig(model="test-model", mock=True)


@pytest.mark.asyncio
async def test_vanilla_loop_simple_completion(mock_api, mock_executor, llm_config):
    """Test that vanilla loop completes when LLM says 'done'."""
    loop = VanillaAgentLoop(mock_api, llm_config, mock_executor)

    # Mock LLM response for completion
    mock_response = MagicMock()
    mock_response.choices = [MagicMock()]
    mock_response.choices[0].message.content = "done Finished successfully"

    with patch("litellm.acompletion", AsyncMock(return_value=mock_response)):
        result = await loop.run("Test task")

    assert result.state == VanillaLoopState.DONE
    assert result.final_output == "Finished successfully"
    assert len(result.turns) == 1


@pytest.mark.asyncio
async def test_vanilla_loop_tool_execution(mock_api, mock_executor, llm_config):
    """Test that vanilla loop executes tools and continues."""
    loop = VanillaAgentLoop(mock_api, llm_config, mock_executor, max_turns=2)

    # Mock LLM responses: tool call then done
    responses = ["skeleton foo.py", "done Done"]

    mock_responses = []
    for r in responses:
        m = MagicMock()
        m.choices = [MagicMock()]
        m.choices[0].message.content = r
        mock_responses.append(m)

    # Mock executor output
    mock_executor.execute.return_value = ("file content", 0, 0)

    with patch("litellm.acompletion", AsyncMock(side_effect=mock_responses)):
        result = await loop.run("Test task")

    assert result.state == VanillaLoopState.DONE
    assert len(result.turns) == 2
    assert result.turns[0].tool_name == "skeleton.format"
    assert result.turns[0].tool_output == "file content"
    assert mock_executor.execute.called


@pytest.mark.asyncio
async def test_vanilla_loop_error_handling(mock_api, mock_executor, llm_config):
    """Test that loop handles tool errors."""
    loop = VanillaAgentLoop(mock_api, llm_config, mock_executor, max_turns=2)

    mock_response = MagicMock()
    mock_response.choices = [MagicMock()]
    mock_response.choices[0].message.content = "skeleton foo.py"

    # Mock executor failure
    mock_executor.execute.side_effect = Exception("Tool failed")

    with patch("litellm.acompletion", AsyncMock(return_value=mock_response)):
        result = await loop.run("Test task")

    # Should continue for max_turns or until failed (here it hits max_turns)
    assert result.state == VanillaLoopState.MAX_TURNS
    assert "Tool failed" in result.turns[0].error
    assert "Error: Tool failed" in result.turns[1].prompt
