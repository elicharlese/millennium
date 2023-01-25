module.exports = /** @type {import('tailwindcss/types/config').Config} */({
	content: [ './src/**/*.{js,jsx,ts,tsx,mdx,md,html,css}' ],
	variants: true,
	corePlugins: {},
	experimental: {
		optimizeUniversalDefaults: true,
		darkMode: true,
	},
	plugins: [
		require('@tailwindcss/typography'),
		require('@tailwindcss/aspect-ratio'),
		function({ addVariant }) {
			addVariant('supports-backdrop-blur', '@supports (backdrop-filter: blur(0)) or (-webkit-backdrop-filter: blur(0))');
			addVariant('demo-dark', '.demo-dark &');
			addVariant('children', '& > *');
		}
	],
	darkMode: 'class'
});
