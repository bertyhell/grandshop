import js from '@eslint/js';
import tseslint from '@electron-toolkit/eslint-config-ts';
import eslintConfigPrettier from '@electron-toolkit/eslint-config-prettier';
import eslintPluginReact from 'eslint-plugin-react';
import eslintPluginReactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';

export default tseslint.config(
	{ ignores: ['**/node_modules', '**/dist', '**/out', 'src/shared/translations/poeditor'] },
	js.configs.recommended,
	tseslint.configs.recommended,
	eslintPluginReact.configs.flat.recommended,
	eslintPluginReact.configs.flat['jsx-runtime'],
	{
		settings: {
			react: {
				version: 'detect',
			},
		},
	},
	{
		files: ['**/*.{ts,tsx}'],
		plugins: {
			'react-hooks': eslintPluginReactHooks,
		},
		rules: {
			'react-hooks/rules-of-hooks': 'error',
		},
	},
	{
		files: ['src/main/**', 'src/preload/**', 'src/scripts/**', '*.config.*'],
		languageOptions: {
			globals: globals.node,
		},
	},
	{
		rules: {
			'@typescript-eslint/explicit-function-return-type': 'off',
		},
	},
	eslintConfigPrettier
);
