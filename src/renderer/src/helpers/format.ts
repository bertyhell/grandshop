export function format(text: string, ...args: any[]) {
	let newText: string = text;
	for (const argIndex in args) {
		newText = newText.replace('{' + argIndex + '}', args[argIndex]);
	}
	return newText;
}
