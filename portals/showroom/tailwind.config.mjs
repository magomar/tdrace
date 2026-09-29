/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        racing: {
          bg: '#0a0c10',
          surface: '#12151d',
          surface2: '#1a1f2c',
          border: '#272d3e',
          text: '#f1f5f9',
          muted: '#94a3b8',
          blue: '#3b82f6',
          cyan: '#06b6d4',
          amber: '#f59e0b',
          red: '#ef4444',
          green: '#10b981',
        },
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
        sans: ['Inter', 'system-ui', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
