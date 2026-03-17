/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        display: ['Inter', 'sans-serif']
      },
      boxShadow: {
        glow: '0 0 40px rgba(148, 163, 184, 0.3)'
      }
    }
  },
  plugins: []
};
