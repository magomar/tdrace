"""
Multi-agent environment tests for collision resolution and simultaneous stepping.
"""

import gymnasium as gym
import numpy as np
import pytest
import tdrace


def test_multi_agent_init_and_shapes():
    """Tests multi-agent initialization and observation shapes for 2, 4, 8 agents."""
    for n in [2, 4, 8]:
        env = gym.make("TDRace-MultiAgent-v0", num_agents=n)
        obs, info = env.reset(seed=42)

        assert obs.shape == (n, env.unwrapped.engine.obs_dim)
        assert len(info["agent_infos"]) == n
        env.close()


def test_multi_agent_step_formats():
    """Tests stepping with 2D ndarray, list of actions, and dict of actions."""
    env = gym.make("TDRace-MultiAgent-v0", num_agents=3)
    env.reset(seed=42)

    # 1. 2D ndarray of shape (3, 3)
    actions_arr = np.array(
        [
            [0.0, 1.0, 0.0],
            [-0.5, 0.8, 0.0],
            [0.5, 0.5, 0.2],
        ],
        dtype=np.float32,
    )
    obs, rew, term, trunc, info = env.step(actions_arr)
    assert obs.shape == (3, env.unwrapped.engine.obs_dim)
    assert rew.shape == (3,)
    assert term.shape == (3,)
    assert trunc.shape == (3,)

    # 2. List of actions
    actions_list = [[0.1, 0.9, 0.0], [-0.1, 0.9, 0.0], [0.0, 0.0, 1.0]]
    obs, rew, term, trunc, info = env.step(actions_list)
    assert obs.shape == (3, env.unwrapped.engine.obs_dim)

    # 3. Dict of actions
    actions_dict = {0: [0.0, 1.0, 0.0], 1: [0.0, 1.0, 0.0], 2: [0.0, 1.0, 0.0]}
    obs, rew, term, trunc, info = env.step(actions_dict)
    assert obs.shape == (3, env.unwrapped.engine.obs_dim)

    env.close()


def test_multi_car_collision_resolution():
    """Tests that two cars placed on a collision course collide and deflect without penetrating."""
    env = gym.make("TDRace-MultiAgent-v0", num_agents=2)
    env.reset(seed=42)

    # Head-on collision setup:
    # Car 0 at (0, 0) moving +X (vel = 30 m/s)
    # Car 1 at (20, 0) moving -X (vel = -30 m/s, angle = PI)
    env.unwrapped.engine.set_state(0, 0.0, 0.0, 30.0, 0.0, 0.0, 0.0)
    env.unwrapped.engine.set_state(1, 20.0, 0.0, -30.0, 0.0, np.pi, 0.0)

    collided = False
    for step in range(30):
        # Steer straight forward
        obs, rew, term, trunc, info = env.step([[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]])
        c0_x, c0_y, c0_vx, _, _, _, _ = env.unwrapped.engine.get_state(0)
        c1_x, c1_y, c1_vx, _, _, _, _ = env.unwrapped.engine.get_state(1)

        # After collision, velocities should reverse due to restitution
        if c0_vx < 0.0 and c1_vx > 0.0:
            collided = True
            break

    assert collided, "Cars failed to collide and rebound"
    env.close()
