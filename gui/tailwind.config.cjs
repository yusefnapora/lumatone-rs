const config = {
  content: [
    "./src/**/*.{html,js,svelte,ts}",

    // skeleton UI
    require('path').join(require.resolve('@brainandbones/skeleton'), '../**/*.{html,js,svelte,ts}'),
  ],

  theme: {
    extend: {},
  },

  darkMode: 'class',

  plugins: [
    // skeleton UI tailwind plugin
    require('@brainandbones/skeleton/tailwind/theme.cjs'),
  ],
};

module.exports = config;
