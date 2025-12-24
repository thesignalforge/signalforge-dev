/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}'
  ],
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: '#0a0e15',
          deep: '#050810',
          surface: '#121820',
          elevated: '#1a2332'
        },
        text: {
          DEFAULT: '#e0e7f1',
          dim: '#7a8a9e'
        },
        cyan: {
          DEFAULT: '#00d9ff',
          bright: '#4dffff',
          dim: '#0099cc',
          glow: 'rgba(0, 217, 255, 0.4)'
        },
        border: '#1f2937'
      },
      fontFamily: {
        mono: ['IBM Plex Mono', 'Menlo', 'Monaco', 'Consolas', 'monospace']
      },
      animation: {
        'pulse-glow': 'pulse-glow 2s ease-in-out infinite',
        'fade-in': 'fade-in 0.3s ease-out'
      },
      keyframes: {
        'pulse-glow': {
          '0%, 100%': { boxShadow: '0 0 5px rgba(0, 217, 255, 0.3)' },
          '50%': { boxShadow: '0 0 20px rgba(0, 217, 255, 0.6)' }
        },
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' }
        }
      }
    }
  },
  plugins: [require('tailwindcss-animate')]
}
