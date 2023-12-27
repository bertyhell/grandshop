import { Dimensions } from '../types';
import { addBorder } from './AddBorder/modification';
import { brightness } from './Brightness/modification';
import { sepia } from './Sepia/modification';
import { resize } from './Resize/modification';
import { annotate } from './Annotate/modification';
import { crop } from './Crop/modification';
import { upscale } from './Upscale/modification';
import { rotate } from './Rotate/modification';
import { mirror } from './Mirror/modification';
import { greyscale } from './Greyscale/modification';

export interface ImageInfo {
	buffer: Buffer;
	dimensions: Dimensions;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type ModificationFunc = (imageInfo: ImageInfo, ...params: any[]) => Promise<Buffer>;

// List of functions that can modify an image by calling GraphicsMagick functions on its gm state
export const modificationFuncs: { [name: string]: ModificationFunc } = {
	addBorder,
	brightness,
	sepia,
	resize,
	annotate,
	crop,
	upscale,
	rotate,
	mirror,
	greyscale,
};
