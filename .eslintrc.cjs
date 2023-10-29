module.exports = {
	extends: [
		'eslint:recommended',
		'plugin:react/jsx-runtime',
		'plugin:react/recommended',
		'plugin:prettier/recommended',
		'prettier',
		'@electron-toolkit/eslint-config-ts/recommended',
		'@electron-toolkit/eslint-config-prettier',
	],
	plugins: ['react', '@typescript-eslint', 'prettier'],
	rules: {
		'@typescript-eslint/explicit-function-return-type': 'off',
	},
};
