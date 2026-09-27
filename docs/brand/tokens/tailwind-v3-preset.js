// Tailwind v3: in tailwind.config.js add  presets: [require('./brand/tokens/tailwind-v3-preset.js')]
module.exports = {
  theme: {
    extend: {
      colors: {
        flame: '#F2611D',
        ember: '#FFC24A',
        ink: '#16140F',
        paper: '#FAF9F5',
        char: '#B8400C',
        smoke: '#57534A',
        ash: '#ECEBE6',
      },
      fontFamily: {
        sans: ['Instrument Sans', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'ui-monospace', 'monospace'],
      },
      borderRadius: { button: '10px', card: '16px', panel: '20px' },
    },
  },
};
