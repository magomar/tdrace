"""
Multi-agent racing environment supporting N simultaneous cars with full physics collision resolution.
"""

from typing import Any, Dict, List, Optional, Tuple, Union
import gymnasium as gym
from gymnasium import spaces
import numpy as np

from ._tdrace import Engine, RewardConfig
from .render import HumanViewer


class TDRaceMultiAgentEnv(gym.Env):
    """
    Multi-Agent TDRace Environment.

    Features:
    - Simultaneous physics stepping for N cars.
    - Full OBB-OBB Separating Axis Theorem (SAT) inter-vehicle collision resolution.
    - Dynamic impulse and momentum exchange on contact.
    - Independent per-agent reward shaping, observation vectors, and lap progress tracking.
    """

    metadata = {
        "render_modes": ["rgb_array", "human"],
        "render_fps": 60,
    }

    def __init__(
        self,
        num_agents: int = 4,
        track_name: str = "classic_grand_prix",
        car_type: str = "sports_car",
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
        return_scalar_signals: bool = False,
    ):
        super().__init__()

        self.num_agents = max(1, num_agents)
        self.track_name = track_name
        self.car_type = car_type
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
        self.return_scalar_signals = return_scalar_signals

        self.engine = Engine(
            track_name=self.track_name,
            num_agents=self.num_agents,
            car_type=self.car_type,
            num_lidar_rays=self.num_lidar_rays,
            max_episode_steps=self.max_episode_steps,
            dt=self.dt,
            reward_config=self.reward_config,
            lap_limit=self.lap_limit,
            terminate_on_off_track=self.terminate_on_off_track,
            terminate_on_wall_crash=self.terminate_on_wall_crash,
        )

        obs_dim = self.engine.obs_dim
        self.single_observation_space = spaces.Box(
            low=-50.0,
            high=50.0,
            shape=(obs_dim,),
            dtype=np.float32,
        )
        self.observation_space = spaces.Box(
            low=-50.0,
            high=50.0,
            shape=(self.num_agents, obs_dim),
            dtype=np.float32,
        )

        self.single_action_space = spaces.Box(
            low=np.array([-1.0, 0.0, 0.0], dtype=np.float32),
            high=np.array([1.0, 1.0, 1.0], dtype=np.float32),
            dtype=np.float32,
        )
        self.action_space = spaces.Box(
            low=np.tile(np.array([-1.0, 0.0, 0.0], dtype=np.float32), (self.num_agents, 1)),
            high=np.tile(np.array([1.0, 1.0, 1.0], dtype=np.float32), (self.num_agents, 1)),
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

        raw_obs_list, info_list = self.engine.reset(seed=seed, randomize_pose=randomize_pose)
        obs = np.asarray(raw_obs_list, dtype=np.float32)

        info = {"agent_infos": info_list}

        if self.render_mode == "human":
            self.render()

        return obs, info

    def step(
        self, actions: Union[np.ndarray, List[Any], Dict[int, Any]]
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, Dict[str, Any]]:
        # Format actions into list of (throttle, steer, brake, handbrake, reverse) tuples
        rust_actions = []

        if isinstance(actions, dict):
            for i in range(self.num_agents):
                act = actions.get(i, [0.0, 0.0, 0.0])
                rust_actions.append(self._parse_single_action(act))
        elif isinstance(actions, (list, tuple)):
            for act in actions:
                rust_actions.append(self._parse_single_action(act))
        elif isinstance(actions, np.ndarray):
            if actions.ndim == 1:
                rust_actions.append(self._parse_single_action(actions))
            else:
                for i in range(actions.shape[0]):
                    rust_actions.append(self._parse_single_action(actions[i]))
        else:
            raise ValueError(f"Unsupported actions type: {type(actions)}")

        # Ensure correct length
        while len(rust_actions) < self.num_agents:
            rust_actions.append((0.0, 0.0, 0.0, False, False))

        obs_list, rew_list, term_list, trunc_list, info_list = self.engine.step_multi(rust_actions)

        obs = np.asarray(obs_list, dtype=np.float32)
        rewards = np.asarray(rew_list, dtype=np.float32)
        terminateds = np.asarray(term_list, dtype=bool)
        truncateds = np.asarray(trunc_list, dtype=bool)
        infos = {
            "agent_infos": info_list,
            "agent_rewards": rewards,
            "agent_terminateds": terminateds,
            "agent_truncateds": truncateds,
        }

        if self.render_mode == "human":
            self.render()

        if self.return_scalar_signals:
            return obs, float(np.mean(rewards)), bool(np.any(terminateds)), bool(np.any(truncateds)), infos

        return obs, rewards, terminateds, truncateds, infos

    @staticmethod
    def _parse_single_action(action: Any) -> Tuple[float, float, float, bool, bool]:
        act = np.nan_to_num(
            np.asarray(action, dtype=np.float32).flatten(),
            nan=0.0,
            posinf=1.0,
            neginf=-1.0,
        )
        if len(act) == 1:
            return 0.5, float(act[0]), 0.0, False, False
        elif len(act) == 2:
            steer = float(act[0])
            gas_brake = float(act[1])
            if gas_brake >= 0.0:
                throttle, brake = gas_brake, 0.0
            else:
                throttle, brake = 0.0, -gas_brake
            return throttle, steer, brake, False, False
        elif len(act) >= 3:
            steer = float(act[0])
            throttle = float(act[1])
            brake = float(act[2])
            handbrake = bool(act[3] > 0.5) if len(act) > 3 else False
            reverse = bool(act[4] > 0.5) if len(act) > 4 else False
            return throttle, steer, brake, handbrake, reverse
        return 0.0, 0.0, 0.0, False, False

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
                    caption=f"TDRace Multi-Agent ({self.num_agents} cars) - {self.track_name}",
                )
            self._viewer.render(rgb_array, fps=self.metadata["render_fps"])
            return None

        return None

    def close(self):
        if self._viewer is not None:
            self._viewer.close()
            self._viewer = None
