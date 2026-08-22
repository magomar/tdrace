"""
Pixel observation and software rasterizer tests.
"""

import gymnasium as gym
import numpy as np
import pytest
import tdrace


def test_pixel_obs_shape_and_channels():
    """Tests default 96x96x3 RGB pixel observation rendering."""
    env = gym.make("TDRace-Pixels-v0")
    obs, info = env.reset(seed=42)

    assert obs.shape == (96, 96, 3)
    assert obs.dtype == np.uint8
    assert np.min(obs) >= 0
    assert np.max(obs) <= 255

    # Check color channels are not all uniform (e.g. green grass != red car != dark asphalt)
    r_channel = obs[:, :, 0]
    g_channel = obs[:, :, 1]
    b_channel = obs[:, :, 2]

    assert not np.array_equal(r_channel, g_channel)
    assert not np.array_equal(g_channel, b_channel)

    # Car is centered in the image and has bright red body [230, 30, 30]
    center_patch = obs[40:56, 40:56, :]
    red_pixels = np.sum((center_patch[:, :, 0] > 180) & (center_patch[:, :, 1] < 100))
    assert red_pixels > 0, "Red car body pixels should be present in center of frame"

    env.close()


@pytest.mark.parametrize("h,w", [(64, 64), (128, 128), (80, 160)])
def test_custom_pixel_resolutions(h, w):
    """Tests custom rasterization resolutions."""
    env = gym.make("TDRace-Pixels-v0", render_width=w, render_height=h)
    obs, _ = env.reset()

    assert obs.shape == (h, w, 3)
    assert obs.dtype == np.uint8

    obs, r, term, trunc, _ = env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))
    assert obs.shape == (h, w, 3)

    env.close()


def test_pixel_rasterizer_skid_marks():
    """Tests that high slip generates visible skid marks in rendered image."""
    env = gym.make("TDRace-Pixels-v0", car_type="drift_car")
    obs, _ = env.reset(seed=42)

    # Accelerate up to speed
    for _ in range(50):
        env.step(np.array([0.0, 1.0, 0.0], dtype=np.float32))

    # Steer hard with handbrake [steer, gas, brake, handbrake]
    for _ in range(25):
        obs, _, _, _, info = env.step(np.array([1.0, 0.6, 0.0, 1.0], dtype=np.float32))

    # Dark skid mark pixels [35, 35, 40] should exist in the frame
    dark_pixels = np.sum((obs[:, :, 0] <= 45) & (obs[:, :, 1] <= 45) & (obs[:, :, 2] <= 50))
    assert dark_pixels > 0, "Skid marks should be visible after handbrake turn"

    env.close()
