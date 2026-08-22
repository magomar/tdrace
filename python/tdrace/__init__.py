"""
TDRace: Ultra-fast deterministic top-down 2D racing simulation engine & Gymnasium environments.
"""

from gymnasium.envs.registration import register

from ._tdrace import Engine, RewardConfig
from .env import (
    TDRaceEnv,
    TDRaceContinuousEnv,
    TDRaceDiscreteEnv,
    TDRaceDriftEnv,
    TDRacePixelsEnv,
)
from .multi_agent import TDRaceMultiAgentEnv
from .render import HumanViewer

# Register Gymnasium environments
register(
    id="TDRace-v0",
    entry_point="tdrace.env:TDRaceEnv",
    max_episode_steps=1000,
    reward_threshold=900.0,
)

register(
    id="TDRace-Continuous-v0",
    entry_point="tdrace.env:TDRaceContinuousEnv",
    max_episode_steps=1000,
    reward_threshold=900.0,
)

register(
    id="TDRace-Discrete-v0",
    entry_point="tdrace.env:TDRaceDiscreteEnv",
    max_episode_steps=1000,
    reward_threshold=900.0,
)

register(
    id="TDRace-Drift-v0",
    entry_point="tdrace.env:TDRaceDriftEnv",
    max_episode_steps=1000,
    reward_threshold=1500.0,
)

register(
    id="TDRace-Pixels-v0",
    entry_point="tdrace.env:TDRacePixelsEnv",
    max_episode_steps=1000,
    reward_threshold=900.0,
)

register(
    id="TDRace-MultiAgent-v0",
    entry_point="tdrace.multi_agent:TDRaceMultiAgentEnv",
    max_episode_steps=1000,
)

__all__ = [
    "Engine",
    "RewardConfig",
    "TDRaceEnv",
    "TDRaceContinuousEnv",
    "TDRaceDiscreteEnv",
    "TDRaceDriftEnv",
    "TDRacePixelsEnv",
    "TDRaceMultiAgentEnv",
    "HumanViewer",
]
