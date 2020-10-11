import { Dimensions } from '../../electron/main-interop';
import { isNil } from 'lodash';
import { CustomError } from '../error';
import { Sharp } from 'sharp';

export async function getImageDimensions(sharp: Sharp): Promise<Dimensions> {
	try {
		const metadata = await sharp.metadata();
		if (!metadata || isNil(metadata.width) || isNil(metadata.height)) {
			throw new CustomError('Failed to fetch dimensions from metadata', null, { metadata });
		}
		return {
			width: metadata.width,
			height: metadata.height,
		};
	} catch (err) {
		throw new CustomError('Failed to get dimensions from image', err);
	}
}
