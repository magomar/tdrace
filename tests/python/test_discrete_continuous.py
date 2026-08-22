"""
Action space tests for continuous and discrete modes.
"""

import gymnasium as gym
import numpy as np
import pytest
import tdrace


def test_discrete_action_space():
    """Tests all discrete actions in TDRace-Discrete-v0."""
    env = gym.make("TDRace-Discrete-v0")
    obs, info = env.reset(seed=42)

    # 0: do nothing
    obs, r, term, trunc, info = env.step(0)
    assert not term

    # 3: accelerate
    speed_init = info["speed_mps"]
    for _ in range(30):
        obs, r, term, trunc, info = env.step(3)
    assert info["speed_mps"] > speed_init

    # 4: brake
    speed_fast = info["speed_mps"]
    for _ in range(30):
        obs, r, term, trunc, info = env.step(4)
    assert info["speed_mps"] < speed_fast

    # 1: steer left
    angle_before = info["normalized_progress"]
    for _ in range(20):
        obs, r, term, trunc, info = env.step(1)

    env.close()


def test_continuous_action_formats():
    """Tests continuous actions: 3D [steer, gas, brake], 2D [steer, gas_brake], and clipping."""
    env = gym.make("TDRace-Continuous-v0")
    env.reset(seed=42)

    # 3D action
    obs, r, term, trunc, info = env.step(np.array([0.2, 0.8, 0.0], dtype=np.float32))
    assert obs.shape == (45,)

    # 2D action [steer, gas_brake]
    obs, r, term, trunc, info = env.step(np.array([-0.5, 0.9], dtype=np.float32))
    assert obs.shape == (45,)

    # 2D reverse/brake [steer, -0.8]
    obs, r, term, trunc, info = env.step(np.array([0.0, -0.8], dtype=np.float32))
    assert obs.shape == (45,)

    # Out of bounds / extreme floats (engine should sanitize and clamp gracefully)
    obs, r, term, trunc, info = env.step(np.array([10.0, 50.0, -10.0], dtype=np.float32))
    assert np.all(np.isfinite(obs))

    env.close()


def test_telemetry_access():
    """Tests detailed vehicle telemetry querying."""
    env = gym.make("TDRace-v0")
    env.reset(seed=42)
    env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))

    telemetry = env.unwrapped.get_telemetry()
    assert "speed_mps" in telemetry
    assert "speed_kmh" in telemetry
    assert "wheel_skid_intensities" in telemetry
    assert "wheel_normal_loads" in telemetry
    assert len(telemetry["wheel_skid_intensities"]) == 4
    assert len(telemetry["wheel_normal_loads"]) == 4

    env.close()
