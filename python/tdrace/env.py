"""
Gymnasium Environment implementations for TDRace.
Compliant with Gymnasium 1.0+ standards.
"""

from typing import Any, Dict, Optional, Tuple, Union
import gymnasium as gym
from gymnasium import spaces
import numpy as np

from ._tdrace import Engine, RewardConfig
from .render import HumanViewer


class TDRaceEnv(gym.Env):
    """
    Standard Single-Agent TDRace Gymnasium Environment.

    Features:
    - High-throughput deterministic 2D top-down physics engine.
    - Vector observations with LIDAR raycasts, vehicle dynamics, spline tracking, and tire telemetry.
    - Software RGB pixel observations matching CarRacing-v3 interface.
    - Configurable tracks, vehicles, and reward shaping.
    """

    metadata = {
        "render_modes": ["rgb_array", "human"],
        "render_fps": 60,
    }

    def __init__(
        self,
        track_name: str = "classic_grand_prix",
        car_type: str = "sports_car",
        obs_type: str = "vector",  # 'vector' or 'pixels'
        action_type: str = "continuous",  # 'continuous' or 'discrete'
        num_lidar_rays: int = 19,
        max_episode_steps: int = 1000,
        dt: float = 1.0 / 60.0,
        reward_config: Optional[RewardConfig] = None,
        lap_limit: int = 1,
        terminate_on_off_track: bool = False,
        terminate_on_wall_crash: bool = False,
        render_mode: Optional[str] = None,
        render_width: int = 96,
        render_height: int = 96,
        randomize_pose_on_reset: bool = False,
    ):
        super().__init__()

        self.track_name = track_name
        self.car_type = car_type
        self.obs_type = obs_type
        self.action_type = action_type
        self.num_lidar_rays = num_lidar_rays
        self.max_episode_steps = max_episode_steps
        self.dt = dt
        self.reward_config = reward_config or RewardConfig.standard_racing()
        self.lap_limit = lap_limit
        self.terminate_on_off_track = terminate_on_off_track
        self.terminate_on_wall_crash = terminate_on_wall_crash
        self.render_mode = render_mode
        self.render_width = render_width
        self.render_height = render_height
        self.randomize_pose_on_reset = randomize_pose_on_reset

        self.engine = Engine(
            track_name=self.track_name,
            num_agents=1,
            car_type=self.car_type,
            num_lidar_rays=self.num_lidar_rays,
            max_episode_steps=self.max_episode_steps,
            dt=self.dt,
            reward_config=self.reward_config,
            lap_limit=self.lap_limit,
            terminate_on_off_track=self.terminate_on_off_track,
            terminate_on_wall_crash=self.terminate_on_wall_crash,
        )

        # Define Observation Space
        if self.obs_type == "pixels":
            self.observation_space = spaces.Box(
                low=0,
                high=255,
                shape=(self.render_height, self.render_width, 3),
                dtype=np.uint8,
            )
        else:
            obs_dim = self.engine.obs_dim
            self.observation_space = spaces.Box(
                low=-50.0,
                high=50.0,
                shape=(obs_dim,),
                dtype=np.float32,
            )

        # Define Action Space
        if self.action_type == "discrete":
            # 5 discrete actions: [do nothing, left, right, gas, brake]
            self.action_space = spaces.Discrete(5)
            self._discrete_actions = [
                (0.0, 0.0, 0.0, False, False),  # 0: do nothing
                (0.0, -1.0, 0.0, False, False),  # 1: steer left
                (0.0, 1.0, 0.0, False, False),  # 2: steer right
                (1.0, 0.0, 0.0, False, False),  # 3: accelerate
                (0.0, 0.0, 0.8, False, False),  # 4: brake
            ]
        else:
            # Continuous: [steer in [-1, 1], throttle in [0, 1], brake in [0, 1]]
            self.action_space = spaces.Box(
                low=np.array([-1.0, 0.0, 0.0], dtype=np.float32),
                high=np.array([1.0, 1.0, 1.0], dtype=np.float32),
                dtype=np.float32,
            )

        self._viewer: Optional[HumanViewer] = None

    def reset(
        self,
        *,
        seed: Optional[int] = None,
        options: Optional[Dict[str, Any]] = None,
    ) -> Tuple[np.ndarray, Dict[str, Any]]:
        super().reset(seed=seed)

        randomize_pose = self.randomize_pose_on_reset
        if options and "randomize_pose" in options:
            randomize_pose = bool(options["randomize_pose"])

        raw_obs, info = self.engine.reset(seed=seed, randomize_pose=randomize_pose)

        if self.obs_type == "pixels":
            obs = self.engine.render_rgb(0, self.render_width, self.render_height, True, None)
            obs = np.asarray(obs, dtype=np.uint8)
        else:
            obs = np.asarray(raw_obs, dtype=np.float32)

        if self.render_mode == "human":
            self.render()

        return obs, info

    def step(
        self, action: Union[int, np.ndarray, list]
    ) -> Tuple[np.ndarray, float, bool, bool, Dict[str, Any]]:
        if self.action_type == "discrete":
            action_idx = int(action)
            throttle, steer, brake, handbrake, reverse = self._discrete_actions[action_idx]
        else:
            act = np.nan_to_num(
                np.asarray(action, dtype=np.float32).flatten(),
                nan=0.0,
                posinf=1.0,
                neginf=-1.0,
            )
            if len(act) == 1:
                steer, throttle, brake, handbrake, reverse = act[0], 0.5, 0.0, False, False
            elif len(act) == 2:
                # [steer, gas_brake]
                steer = float(act[0])
                gas_brake = float(act[1])
                if gas_brake >= 0.0:
                    throttle = gas_brake
                    brake = 0.0
                else:
                    throttle = 0.0
                    brake = -gas_brake
                handbrake = False
                reverse = False
            elif len(act) >= 3:
                # [steer, throttle, brake, (handbrake), (reverse)]
                steer = float(act[0])
                throttle = float(act[1])
                brake = float(act[2])
                handbrake = bool(act[3] > 0.5) if len(act) > 3 else False
                reverse = bool(act[4] > 0.5) if len(act) > 4 else False
            else:
                steer, throttle, brake, handbrake, reverse = 0.0, 0.0, 0.0, False, False

        raw_obs, reward, terminated, truncated, info = self.engine.step_single(
            throttle=throttle,
            steer=steer,
            brake=brake,
            handbrake=handbrake,
            reverse=reverse,
        )

        if self.obs_type == "pixels":
            obs = self.engine.render_rgb(0, self.render_width, self.render_height, True, None)
            obs = np.asarray(obs, dtype=np.uint8)
        else:
            obs = np.asarray(raw_obs, dtype=np.float32)

        if self.render_mode == "human":
            self.render()

        return obs, float(reward), bool(terminated), bool(truncated), info

    def render(self) -> Optional[np.ndarray]:
        if self.render_mode is None:
            return None

        rgb = self.engine.render_rgb(0, self.render_width, self.render_height, True, None)
        rgb_array = np.asarray(rgb, dtype=np.uint8)

        if self.render_mode == "rgb_array":
            return rgb_array
        elif self.render_mode == "human":
            if self._viewer is None:
                self._viewer = HumanViewer(
                    width=512,
                    height=512,
                    caption=f"TDRace - {self.track_name}",
                )
            self._viewer.render(rgb_array, fps=self.metadata["render_fps"])
            return None

        return None

    def close(self):
        if self._viewer is not None:
            self._viewer.close()
            self._viewer = None

    def get_telemetry(self) -> Dict[str, Any]:
        """Returns deep real-time vehicle telemetry."""
        return self.engine.get_telemetry(0)

    def set_state(
        self,
        x: float,
        y: float,
        vx: float,
        vy: float,
        angle: float,
        angular_velocity: float,
    ):
        """Sets vehicle pose and velocity."""
        self.engine.set_state(0, x, y, vx, vy, angle, angular_velocity)

    def get_state(self) -> Tuple[float, float, float, float, float, float, float]:
        """Returns (x, y, vx, vy, angle, angular_velocity, steer_angle)."""
        return self.engine.get_state(0)


class TDRaceContinuousEnv(TDRaceEnv):
    """TDRace with continuous action space [steer, throttle, brake] and vector observations."""

    def __init__(self, **kwargs):
        kwargs["action_type"] = "continuous"
        kwargs["obs_type"] = "vector"
        super().__init__(**kwargs)


class TDRaceDiscreteEnv(TDRaceEnv):
    """TDRace with 5-action discrete action space and vector observations."""

    def __init__(self, **kwargs):
        kwargs["action_type"] = "discrete"
        kwargs["obs_type"] = "vector"
        super().__init__(**kwargs)


class TDRaceDriftEnv(TDRaceEnv):
    """TDRace tuned for drift scoring on Drift Park circuit with drift machine."""

    def __init__(self, **kwargs):
        kwargs.setdefault("track_name", "drift_park")
        kwargs.setdefault("car_type", "drift_car")
        kwargs.setdefault("reward_config", RewardConfig.drift_challenge())
        kwargs.setdefault("action_type", "continuous")
        kwargs.setdefault("obs_type", "vector")
        super().__init__(**kwargs)


class TDRacePixelsEnv(TDRaceEnv):
    """
    TDRace with top-down 96x96x3 RGB pixel observations and continuous actions.
    Drop-in replacement for Gymnasium CarRacing-v3 with >50x throughput.
    """

    def __init__(self, **kwargs):
        kwargs["obs_type"] = "pixels"
        kwargs.setdefault("render_width", 96)
        kwargs.setdefault("render_height", 96)
        kwargs.setdefault("action_type", "continuous")
        super().__init__(**kwargs)
