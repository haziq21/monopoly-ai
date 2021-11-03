const colors = require('tailwindcss/colors');

const config = {
	mode: 'jit',

	purge: ['./src/**/*.{html,js,svelte,ts}'],
	// content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {},
		colors: {
			gray: colors.slate,
			red: colors.red,
			blue: colors.indigo,
			yellow: colors.amber,
		},
	},

	plugins: []
};

module.exports = config;
