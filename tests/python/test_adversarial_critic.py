"""
Adversarial stress test suite for TDRace Headless Engine & Gymnasium API Bindings.
"""

import math
import numpy as np
import pytest
import gymnasium as gym
from gymnasium.utils.env_checker import check_env
import tdrace
from tdrace import Engine, RewardConfig


# ==============================================================================
# 1. Strict Gymnasium API Compliance on all registered environments
# ==============================================================================

@pytest.mark.parametrize(
    "env_id,kwargs",
    [
        ("TDRace-v0", {}),
        ("TDRace-Continuous-v0", {}),
        ("TDRace-Discrete-v0", {}),
        ("TDRace-Drift-v0", {}),
        ("TDRace-Pixels-v0", {}),
        ("TDRace-MultiAgent-v0", {"num_agents": 2, "return_scalar_signals": True}),
        ("TDRace-MultiAgent-v0", {"num_agents": 4, "return_scalar_signals": True}),
    ],
)
def test_all_registered_envs_strict_check_env(env_id, kwargs):
    """Runs gymnasium check_env on every single registered environment variation."""
    env = gym.make(env_id, **kwargs)
    # Check unwrapped to ensure API compliance
    check_env(env.unwrapped)
    env.close()


# ==============================================================================
# 2. Adversarial Action Input Stress Testing (NaN, Inf, Overflow, Type Mismatches)
# ==============================================================================

@pytest.mark.parametrize(
    "env_id",
    ["TDRace-v0", "TDRace-Continuous-v0", "TDRace-Pixels-v0"],
)
def test_extreme_actions_continuous_no_crashes(env_id):
    """Feeds extreme, malicious, and corrupt action inputs into continuous environments."""
    env = gym.make(env_id)
    obs, info = env.reset(seed=42)

    adversarial_actions = [
        # NaNs and Infs
        np.array([np.nan, np.nan, np.nan], dtype=np.float32),
        np.array([np.inf, -np.inf, np.inf], dtype=np.float32),
        np.array([np.nan, 1.0, 0.0], dtype=np.float32),
        np.array([0.0, np.nan, 0.0], dtype=np.float32),
        np.array([0.0, 0.0, np.nan], dtype=np.float32),
        # Astronomical values
        np.array([1e30, -1e30, 1e30], dtype=np.float32),
        np.array([-1e30, 1e30, -1e30], dtype=np.float32),
        # Denormals / Subnormals
        np.array([1e-40, -1e-40, 1e-40], dtype=np.float32),
        # Out of bounds
        np.array([999.0, 999.0, 999.0], dtype=np.float32),
        np.array([-999.0, -999.0, -999.0], dtype=np.float32),
        # Varied shapes
        np.array([], dtype=np.float32),
        np.array([0.5], dtype=np.float32),
        np.array([0.5, 0.5], dtype=np.float32),
        np.array([0.5, 0.5, 0.5, 1.0], dtype=np.float32),
        np.array([0.5, 0.5, 0.5, 1.0, 1.0], dtype=np.float32),
        np.array([0.5, 0.5, 0.5, 0.0, 0.0, 9.9, 8.8], dtype=np.float32),
        # Python lists and tuples
        [float("nan"), 1.0, 0.0],
        [float("inf"), float("-inf"), 0.0],
        (0.0, 0.0, 0.0),
        [],
    ]

    for i, act in enumerate(adversarial_actions):
        # Must not crash, segfault, or panic
        obs, rew, term, trunc, info = env.step(act)
        assert obs is not None
        assert np.all(np.isfinite(obs)), f"Obs contained NaN/Inf on adversarial action #{i}: {act}"
        assert math.isfinite(rew), f"Reward was not finite on action #{i}: {act}"
        assert isinstance(term, bool)
        assert isinstance(trunc, bool)
        assert isinstance(info, dict)

    env.close()


def test_discrete_env_out_of_bounds_action():
    """Tests discrete environment with out of range action integers."""
    env = gym.make("TDRace-Discrete-v0")
    env.reset(seed=42)

    # Valid actions are 0..4. If out of bounds integer is supplied, it should either raise or handle gracefully
    for act in [-10, 5, 100, 9999]:
        try:
            obs, rew, term, trunc, info = env.step(act)
            assert obs is not None
        except (IndexError, ValueError, KeyError):
            pass  # Raising standard python exception is acceptable for discrete space bounds

    env.close()


# ==============================================================================
# 3. Determinism and Multi-Seed Reproducibility
# ==============================================================================

@pytest.mark.parametrize("seed", [0, 1, 42, 1337, 999999])
def test_reproducibility_across_multiple_seeds(seed):
    """Ensures bit-exact replay for arbitrary seeds across independent engine instances."""
    env1 = gym.make("TDRace-v0")
    env2 = gym.make("TDRace-v0")

    obs1, info1 = env1.reset(seed=seed)
    obs2, info2 = env2.reset(seed=seed)

    np.testing.assert_array_equal(obs1, obs2, err_msg=f"Seed {seed} reset mismatch")

    rng = np.random.RandomState(seed)
    for step in range(100):
        action = rng.uniform(low=[-1.0, 0.0, 0.0], high=[1.0, 1.0, 1.0])
        o1, r1, t1, tr1, inf1 = env1.step(action)
        o2, r2, t2, tr2, inf2 = env2.step(action)

        np.testing.assert_array_equal(o1, o2, err_msg=f"Step {step} obs mismatch with seed {seed}")
        assert r1 == r2, f"Step {step} reward mismatch with seed {seed}"
        assert t1 == t2, f"Step {step} term mismatch with seed {seed}"
        assert tr1 == tr2, f"Step {step} trunc mismatch with seed {seed}"

    env1.close()
    env2.close()


def test_seed_randomize_pose_diversity():
    """Verifies that randomize_pose=True generates distinct initial poses across distinct seeds."""
    env = gym.make("TDRace-v0", randomize_pose_on_reset=True)
    obs1, _ = env.reset(seed=100)
    obs2, _ = env.reset(seed=200)
    obs3, _ = env.reset(seed=300)

    # Initial positions/angles must differ
    assert not np.allclose(obs1, obs2)
    assert not np.allclose(obs1, obs3)

    # But resetting with the same seed must produce the exact same randomized pose
    obs1_again, _ = env.reset(seed=100)
    np.testing.assert_array_equal(obs1, obs1_again)
    env.close()


# ==============================================================================
# 4. Physics Edge Cases & Extreme State Manipulation
# ==============================================================================

def test_teleport_car_supersonic_speed():
    """Teleports car with supersonic speed (300 m/s ~ 1080 km/h) and verifies stability."""
    env = gym.make("TDRace-v0")
    env.reset(seed=42)

    # Teleport to 300 m/s
    env.unwrapped.set_state(0.0, 0.0, 300.0, 0.0, 0.0, 0.0)
    obs, rew, term, trunc, info = env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))

    assert np.all(np.isfinite(obs))
    assert math.isfinite(rew)
    telemetry = env.unwrapped.get_telemetry()
    assert math.isfinite(telemetry["speed_mps"])
    assert math.isfinite(telemetry["speed_kmh"])
    env.close()


def test_teleport_car_deep_inside_wall():
    """Teleports car directly inside a wall barrier to test collision penetration resolution."""
    env = gym.make("TDRace-v0")
    env.reset(seed=42)

    # Teleport right onto track boundary wall
    env.unwrapped.set_state(25.0, 0.0, 20.0, 0.0, 0.0, 0.0)
    for _ in range(20):
        obs, rew, term, trunc, info = env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))
        assert np.all(np.isfinite(obs))
        assert math.isfinite(rew)

    env.close()


def test_teleport_car_far_off_world():
    """Teleports car 10,000 meters into the void."""
    env = gym.make("TDRace-v0")
    env.reset(seed=42)

    env.unwrapped.set_state(10000.0, 10000.0, 0.0, 0.0, 0.0, 0.0)
    obs, rew, term, trunc, info = env.step(np.array([0.0, 0.0, 0.0], dtype=np.float32))

    assert np.all(np.isfinite(obs))
    assert math.isfinite(rew)
    assert info["is_off_track"] == True

    # Render rgb at 10,000m
    rgb = env.unwrapped.render()
    # If render_mode is None it returns None, so test with rgb_array
    env.close()

    env_pix = gym.make("TDRace-Pixels-v0")
    env_pix.reset(seed=42)
    env_pix.unwrapped.set_state(10000.0, 10000.0, 0.0, 0.0, 0.0, 0.0)
    pix_obs, r, _, _, _ = env_pix.step(np.array([0.0, 0.0, 0.0], dtype=np.float32))
    assert pix_obs.shape == (96, 96, 3)
    assert np.all(np.isfinite(pix_obs))
    env_pix.close()


# ==============================================================================
# 5. Reward Function Adversarial Anti-Exploit Tests
# ==============================================================================

def test_anti_exploit_stationary_donut_spinning():
    """
    Ensures that an agent spinning donuts on the spot does NOT accumulate
    unbounded positive reward compared to forward driving.
    """
    env = gym.make("TDRace-v0")
    env.reset(seed=42)

    # Donut spinning: full steer + handbrake + full throttle
    donut_reward = 0.0
    for _ in range(300):
        obs, rew, term, trunc, info = env.step(np.array([1.0, 1.0, 0.0, 1.0], dtype=np.float32))
        donut_reward += rew

    env.reset(seed=42)
    # Forward driving
    forward_reward = 0.0
    for _ in range(300):
        obs, rew, term, trunc, info = env.step(np.array([0.0, 1.0, 0.0, 0.0], dtype=np.float32))
        forward_reward += rew

    # Forward driving must earn vastly more reward than stationary donuts in standard racing
    assert forward_reward > donut_reward * 2.0, (
        f"Forward reward ({forward_reward}) should dominate donut reward ({donut_reward})"
    )
    env.close()


def test_anti_exploit_wall_grinding_penalty():
    """Ensures that driving into and scraping along walls applies substantial penalties."""
    env = gym.make("TDRace-v0")
    env.reset(seed=42)

    # Ram into outer barrier / wall
    wall_reward = 0.0
    wall_hits = 0
    for _ in range(120):
        obs, rew, term, trunc, info = env.step(np.array([-0.9, 1.0, 0.0], dtype=np.float32))
        wall_reward += rew
        if info["wall_hit"]:
            wall_hits += 1

    assert wall_hits > 0, "Wall collisions should have occurred"
    # Net wall grinding reward should be heavily penalized / negative
    assert wall_reward < 0.0, f"Wall scraping generated positive reward: {wall_reward}"
    env.close()


# ==============================================================================
# 6. Multi-Agent Stress Testing (16 cars, dense pileup)
# ==============================================================================

def test_multi_agent_extreme_car_density():
    """Tests 16 cars all spawned simultaneously in dense proximity."""
    env = gym.make("TDRace-MultiAgent-v0", num_agents=16, track_name="classic_grand_prix")
    obs, info = env.reset(seed=42)

    assert obs.shape == (16, env.unwrapped.engine.obs_dim)
    assert len(info["agent_infos"]) == 16

    # Step 100 steps of full throttle for all 16 cars
    actions = np.zeros((16, 3), dtype=np.float32)
    actions[:, 1] = 1.0  # full gas

    for step in range(100):
        obs, rews, terms, truncs, infos = env.step(actions)
        assert obs.shape == (16, env.unwrapped.engine.obs_dim)
        assert len(rews) == 16
        assert np.all(np.isfinite(obs))
        assert np.all(np.isfinite(rews))

    env.close()
