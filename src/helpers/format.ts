export function format(text: string, ...args: any[]) {
  for (const argIndex in args) {
    text = text.replace("{" + argIndex + "}", args[argIndex])
  }
  return text;
}
