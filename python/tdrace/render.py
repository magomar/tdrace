"""
Rendering and human display visualizer for TDRace.
Provides high-performance Pygame display when render_mode='human'.
"""

from typing import Optional
import numpy as np


class HumanViewer:
    """Pygame-based window visualizer for human interaction and real-time viewing."""

    def __init__(self, width: int = 512, height: int = 512, caption: str = "TDRace"):
        self.width = width
        self.height = height
        self.caption = caption
        self._pygame = None
        self._screen = None
        self._clock = None
        self._is_open = False

    def _lazy_init(self):
        if not self._is_open:
            import pygame

            pygame.init()
            pygame.display.init()
            self._pygame = pygame
            self._screen = pygame.display.set_mode((self.width, self.height))
            pygame.display.set_caption(self.caption)
            self._clock = pygame.time.Clock()
            self._is_open = True

    def render(self, rgb_array: np.ndarray, fps: int = 60) -> bool:
        """Renders an RGB ndarray (H, W, 3) to the window."""
        self._lazy_init()
        pygame = self._pygame

        # Handle window events
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                self.close()
                return False

        # Scale RGB image to window size
        # Pygame surfarray expects (W, H, 3)
        h, w, c = rgb_array.shape
        surf = pygame.surfarray.make_surface(np.transpose(rgb_array, (1, 0, 2)))
        if (w, h) != (self.width, self.height):
            surf = pygame.transform.scale(surf, (self.width, self.height))

        self._screen.blit(surf, (0, 0))
        pygame.display.flip()
        self._clock.tick(fps)
        return True

    def close(self):
        """Closes the Pygame display window."""
        if self._is_open:
            if self._pygame is not None:
                self._pygame.display.quit()
                self._pygame.quit()
            self._is_open = False
            self._screen = None
            self._pygame = None
