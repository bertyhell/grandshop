import util from 'util';
import type { Sharp } from 'sharp';
import type Upscaler from 'upscaler/node';

import ImageManager from '../image-manager';
import { ImageInfo } from '../modifications';

// @tensorflow/tfjs-node still uses util functions that were removed in node 23
const legacyUtil = util as typeof util & {
	isNullOrUndefined?: (value: unknown) => boolean;
	isArray?: (value: unknown) => boolean;
};
legacyUtil.isNullOrUndefined ??= (value: unknown) => value === null || value === undefined;
legacyUtil.isArray ??= Array.isArray;

// Loading tensorflow and the model is slow, so only load them when the first upscale is requested
let upscalerPromise: Promise<{
	upscaler: Upscaler;
	encodePng: (tensor: import('@tensorflow/tfjs-node').Tensor3D) => Promise<Uint8Array>;
}> | null = null;

function getUpscaler() {
	upscalerPromise ??= (async () => {
		// Use require, since the esm builds of these packages cannot be loaded by node
		/* eslint-disable @typescript-eslint/no-require-imports */
		const UpscalerClass: typeof Upscaler = require('upscaler/node');
		const model = require('@upscalerjs/esrgan-thick/4x');
		const tfNode: typeof import('@tensorflow/tfjs-node') = require('@tensorflow/tfjs-node');
		/* eslint-enable @typescript-eslint/no-require-imports */
		return {
			upscaler: new UpscalerClass({ model }),
			encodePng: tfNode.node.encodePng,
		};
	})();
	return upscalerPromise;
}

export async function upscale(imageInfo: ImageInfo): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);
	const { upscaler, encodePng } = await getUpscaler();

	const imageThreeChannels = await sharp.removeAlpha().png().toBuffer();

	const tensor = await upscaler.upscale(imageThreeChannels, {
		output: 'tensor',
	});
	const upscaledTensor = await encodePng(tensor);
	tensor.dispose();
	return Buffer.from(upscaledTensor);
}
