import ImageManager from '../image-manager';
import { ImageInfo } from '../modifications';
import { Sharp } from 'sharp';
import Upscaler from '../../../../node_modules/upscaler/dist/node-gpu/upscalerjs/src/node-gpu/index.js';
import model from '../../../../node_modules/@upscalerjs/esrgan-thick/dist/cjs/models/esrgan-thick/src/x4/index.js';
import { node as tfNode } from '@tensorflow/tfjs-node-gpu';

export async function upscale(imageInfo: ImageInfo): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const upscaler = new Upscaler({
		model,
	});

	const imageThreeChannels = await sharp.removeAlpha().toBuffer();

	const tensor = await upscaler.upscale(imageThreeChannels, {
		output: 'tensor',
	});
	const upscaledTensor = await tfNode.encodePng(tensor);
	tensor.dispose();
	return Buffer.from(upscaledTensor);
}
