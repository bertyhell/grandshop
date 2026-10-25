// The node entrypoint of upscaler ships without type declarations
declare module 'upscaler/node' {
	import type { Tensor3D } from '@tensorflow/tfjs-node';

	export default class Upscaler {
		constructor(options: { model: unknown });
		upscale(image: Buffer, options: { output: 'tensor' }): Promise<Tensor3D>;
		dispose(): Promise<void>;
	}
}
