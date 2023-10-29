export type RendererCallFunction =
	| 'setOriginalImage'
	| 'setOriginalImageDimensions'
	| 'setModifiedImage'
	| 'setModifiedImageDimensions'
	| 'setModificationInProgress'
	| 'showNotification'
	| 'setColorAtPixel';

export interface RendererCallFunctionRequest {
	func: RendererCallFunction;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	funcArgs: any[];
}

export type ElectronCallFunction =
	| 'openImage'
	| 'applyModifications'
	| 'saveImage'
	| 'setLanguage'
	| 'getColorAtPixel';

export interface ElectronCallFunctionRequest {
	func: ElectronCallFunction;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	funcArgs: any[];
}

export interface FrontendImageInfo {
	base64: string;
	dimensions: Dimensions;
}

export interface Dimensions {
	width: number;
	height: number;
}

export enum Extension {
	jpg = 'jpg',
	png = 'png',
	gif = 'gif',
	bmp = 'bmp',
	tif = 'tif',
	jpeg = 'jpeg',
	webp = 'webp',
	svg = 'svg',
}
