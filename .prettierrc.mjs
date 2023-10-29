export default {
	useTabs: true,
	tabWidth: 4,
	trailingComma: 'es5',
	singleQuote: true,
	endOfLine: 'auto',
	printWidth: 100,
	overrides: [
		{
			files: ['**/*.css', '**/*.scss', '**/*.html'],
			options: {
				singleQuote: false,
				printWidth: 1000,
			},
		},
		{
			files: ['package.json', 'package-lock.json', 'src/shared/translations/nl.json'],
			options: {
				useTabs: false,
				tabWidth: 2,
			},
		},
	],
};
