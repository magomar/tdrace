"""
Official circuit selection tests (spec 042): the engine accepts any official circuit id or alias.
"""

import numpy as np
import pytest
from tdrace.env import TDRaceEnv


@pytest.mark.parametrize(
    ("track_name", "expected_name"),
    [
        ("monza", "Monza Autodromo Nazionale"),
        ("drift_park", "Ridge Ring"),
        ("daytona", "Daytona International Speedway"),
        ("holjes_rx", "Höljes Motorstadion"),
        ("oval", "Tri-Oval Speedway"),
        ("no_such_circuit", "Coastal Grand Prix"),
    ],
)
def test_env_runs_on_official_circuit(track_name, expected_name):
    env = TDRaceEnv(track_name=track_name)
    assert env.engine.track_name == expected_name
    obs, _info = env.reset(seed=1)
    assert np.all(np.isfinite(obs))
    for _ in range(10):
        obs, _reward, _term, _trunc, _info = env.step(env.action_space.sample())
    assert np.all(np.isfinite(obs))
    env.close()
