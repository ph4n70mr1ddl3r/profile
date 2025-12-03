import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./src/app/**/*.{js,ts,jsx,tsx}', './src/components/**/*.{js,ts,jsx,tsx}', './src/lib/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        trust: {
          default: '#2563eb',
          subtle: '#eff6ff',
          warning: '#f59e0b',
          danger: '#dc2626'
        }
      }
    }
  },
  plugins: []
};

export default config;
