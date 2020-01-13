import { State } from 'gm';
import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function crop(imageInfo: ImageInfo, top: number, right: number, bottom: number, left: number): Promise<Buffer> {
	const gmState: State = ImageManager.bufferToGm(imageInfo.buffer);

	const state = gmState.crop(
		imageInfo.dimensions.width - right - left,
		imageInfo.dimensions.height - top - bottom,
		left,
		top
	);

	return ImageManager.gmToBuffer(state);
}
