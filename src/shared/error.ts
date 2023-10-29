import i18n from './translations/i18n';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function replacer(key: string, value: any): any {
	if (key === 'request') {
		// Avoid huge request object in error json
		return '[request]';
	}
	return value;
}

export class CustomError {
	public message: string;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	public innerException: any | null;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	public additionalInfo: any | null;
	public name: string = 'Error';
	public stack: string;
	public timestamp: string = new Date().toISOString();

	constructor(
		message: string = i18n.t('shared/error___something-went-wrong'),
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		innerException: any = null,
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		additionalInfo: any = null
	) {
		this.message = message;
		this.innerException = innerException;
		this.additionalInfo = JSON.stringify(additionalInfo, replacer, 2);
		Error.captureStackTrace(this, this.constructor);

		if (innerException && typeof innerException.stack === 'string') {
			this.stack = innerException.stack;
		} else {
			this.stack = new Error().stack || '';
		}
	}

	public toString(): string {
		return JSON.stringify(this, replacer, 2);
	}
}
