/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: ["./src/**/*.{rs,html}"],
  theme: {
    extend: {
      colors: {
        claude: {
          DEFAULT: '#D97757',
          dark: '#C4684B',
          light: '#F0B3A0',
          50: '#FEF3EF',
        },
      },
    },
  },
  plugins: [require('@tailwindcss/typography')],
}
