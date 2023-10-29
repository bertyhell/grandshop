import i18n from './translations/i18n';

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
		this.additionalInfo = additionalInfo;
		Error.captureStackTrace(this, this.constructor);

		if (innerException && typeof innerException.stack === 'string') {
			this.stack = innerException.stack;
		} else {
			this.stack = new Error().stack || '';
		}
	}

	public toString(): string {
		return JSON.stringify(
			this,
			(key, value) => {
				if (key === 'request') {
					// Avoid huge request object in error json
					return '[request]';
				}
				return value;
			},
			2
		);
	}
}
