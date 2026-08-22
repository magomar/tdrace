"""
Determinism and reproducibility tests for TDRace.
"""

import gymnasium as gym
import numpy as np
import pytest
import tdrace


def test_seed_determinism_vector():
    """Verifies that two runs with the same seed and action sequence produce bit-exact results."""
    env1 = gym.make("TDRace-v0")
    env2 = gym.make("TDRace-v0")

    obs1, info1 = env1.reset(seed=42)
    obs2, info2 = env2.reset(seed=42)

    np.testing.assert_array_equal(obs1, obs2)

    # Deterministic pseudo-random action sequence
    rng = np.random.RandomState(999)
    actions = [rng.uniform(low=[-1.0, 0.0, 0.0], high=[1.0, 1.0, 1.0]) for _ in range(150)]

    for step, act in enumerate(actions):
        next_obs1, r1, term1, trunc1, inf1 = env1.step(act)
        next_obs2, r2, term2, trunc2, inf2 = env2.step(act)

        np.testing.assert_array_equal(
            next_obs1,
            next_obs2,
            err_msg=f"Observation mismatch at step {step}",
        )
        assert r1 == r2, f"Reward mismatch at step {step}: {r1} != {r2}"
        assert term1 == term2, f"Terminated mismatch at step {step}"
        assert trunc1 == trunc2, f"Truncated mismatch at step {step}"

    env1.close()
    env2.close()


def test_seed_determinism_pixels():
    """Verifies that pixel observations are identical with identical seeds and actions."""
    env1 = gym.make("TDRace-Pixels-v0")
    env2 = gym.make("TDRace-Pixels-v0")

    obs1, _ = env1.reset(seed=123)
    obs2, _ = env2.reset(seed=123)

    np.testing.assert_array_equal(obs1, obs2)

    # 50 steps of acceleration
    action = np.array([0.1, 1.0, 0.0], dtype=np.float32)
    for step in range(50):
        obs1, r1, _, _, _ = env1.step(action)
        obs2, r2, _, _, _ = env2.step(action)
        np.testing.assert_array_equal(obs1, obs2, err_msg=f"Pixel mismatch at step {step}")
        assert r1 == r2

    env1.close()
    env2.close()


def test_state_save_and_restore():
    """Tests save, modify, and restore state via get_state() and set_state()."""
    env = gym.make("TDRace-v0")
    env.reset(seed=42)

    # Step forward 40 steps
    for _ in range(40):
        env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))

    saved_state = env.unwrapped.get_state()
    x, y, vx, vy, angle, ang_vel, steer = saved_state
    speed = np.hypot(vx, vy)
    assert speed > 2.0

    # Step another 30 steps with steering
    for _ in range(30):
        env.step(np.array([0.5, 1.0, 0.0], dtype=np.float32))

    mod_state = env.unwrapped.get_state()
    assert mod_state != saved_state

    # Restore state
    env.unwrapped.set_state(x, y, vx, vy, angle, ang_vel)
    restored_state = env.unwrapped.get_state()

    assert np.isclose(restored_state[0], saved_state[0], atol=1e-4)
    assert np.isclose(restored_state[1], saved_state[1], atol=1e-4)
    assert np.isclose(restored_state[2], saved_state[2], atol=1e-4)
    assert np.isclose(restored_state[3], saved_state[3], atol=1e-4)
    assert np.isclose(restored_state[4], saved_state[4], atol=1e-4)

    env.close()
