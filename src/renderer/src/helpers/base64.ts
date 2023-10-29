export function getImageSource(base64: string | null | undefined): string {
	return 'data:image/png;base64,' + (base64 || '');
}
