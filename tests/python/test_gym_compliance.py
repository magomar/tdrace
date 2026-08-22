"""
Gymnasium API compliance tests using gymnasium.utils.env_checker.check_env.
"""

import gymnasium as gym
from gymnasium.utils.env_checker import check_env
import pytest
import numpy as np
import tdrace


@pytest.mark.parametrize(
    "env_id",
    [
        "TDRace-v0",
        "TDRace-Continuous-v0",
        "TDRace-Discrete-v0",
        "TDRace-Drift-v0",
        "TDRace-Pixels-v0",
    ],
)
def test_gym_check_env_single_agent(env_id):
    """Verifies that all single-agent environments strictly comply with Gymnasium standard."""
    env = gym.make(env_id)
    check_env(env.unwrapped)
    env.close()


def test_gym_check_env_multi_agent():
    """Verifies multi-agent env compatibility with Gymnasium check_env."""
    env = gym.make("TDRace-MultiAgent-v0", num_agents=4, return_scalar_signals=True)
    check_env(env.unwrapped)
    env.close()


@pytest.mark.parametrize(
    "env_id,expected_obs_shape,expected_obs_dtype",
    [
        ("TDRace-v0", (45,), np.float32),
        ("TDRace-Continuous-v0", (45,), np.float32),
        ("TDRace-Discrete-v0", (45,), np.float32),
        ("TDRace-Drift-v0", (45,), np.float32),
        ("TDRace-Pixels-v0", (96, 96, 3), np.uint8),
    ],
)
def test_observation_space_shapes_and_types(env_id, expected_obs_shape, expected_obs_dtype):
    env = gym.make(env_id)
    obs, info = env.reset()
    assert obs.shape == expected_obs_shape
    assert obs.dtype == expected_obs_dtype
    assert env.observation_space.contains(obs)
    assert isinstance(info, dict)
    env.close()


def test_render_rgb_array():
    env = gym.make("TDRace-v0", render_mode="rgb_array", render_width=128, render_height=128)
    obs, info = env.reset()
    frame = env.render()
    assert frame is not None
    assert frame.shape == (128, 128, 3)
    assert frame.dtype == np.uint8
    assert np.any(frame > 0)
    env.close()


def test_render_none_when_unspecified():
    env = gym.make("TDRace-v0", render_mode=None)
    env.reset()
    frame = env.render()
    assert frame is None
    env.close()
