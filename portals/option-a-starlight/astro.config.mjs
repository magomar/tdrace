import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';

export default defineConfig({
  integrations: [
    starlight({
      title: 'TdRace Technical Reference',
      description: 'Engineering specifications, Pacejka physics, 25 vehicle classes, and 90 circuit directories.',
      social: {
        github: 'https://github.com/magomar/tdrace',
      },
      customCss: ['./src/styles/custom.css'],
      sidebar: [
        {
          label: 'Master Index',
          items: [{ label: 'Knowledge Base Overview', link: '/' }],
        },
        {
          label: 'Simulation Physics',
          autogenerate: { directory: 'physics' },
        },
        {
          label: 'Vehicle Rosters',
          autogenerate: { directory: 'vehicles' },
        },
        {
          label: 'Circuit Directory',
          autogenerate: { directory: 'circuits' },
        },
        {
          label: 'Engineering Specs',
          autogenerate: { directory: 'engineering' },
        },
      ],
    }),
  ],
  markdown: {
    remarkPlugins: [remarkMath],
    rehypePlugins: [rehypeKatex],
  },
});
