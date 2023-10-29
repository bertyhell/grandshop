export function format(text: string, ...args: (string | number)[]) {
	let newText: string = text;
	for (const argIndex in args) {
		newText = newText.replace('{' + argIndex + '}', String(args[argIndex]));
	}
	return newText;
}
