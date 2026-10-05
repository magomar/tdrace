import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';

export default defineConfig({
  integrations: [tailwind()],
  // Showroom routes from before the Codex (spec 087).
  redirects: {
    '/vehicles': '/showroom/cars',
    '/circuits': '/showroom/circuits',
    '/physics': '/technical/physics-lab',
  },
});
