import { addBorder } from './AddBorder/modification';
import { brightness } from './Brightness/modification';
import { sepia } from './Sepia/modification';
import { resize } from './Resize/modification';
import { annotate } from './Annotate/modification';
import { crop } from './Crop/modification';
import { Dimensions } from '../../electron/main-interop';

export interface ImageInfo {
	buffer: Buffer;
	dimensions: Dimensions;
}

export type ModificationFunc = (imageInfo: ImageInfo, ...params: any[]) => Promise<Buffer>;

// List of functions that can modify an image by calling GraphicsMagick functions on its gm state
const modificationFuncs: { [name: string]: ModificationFunc } = {
	addBorder,
	brightness,
	sepia,
	resize,
	annotate,
	crop,
};

export default modificationFuncs;
