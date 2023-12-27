import { FrontendImageInfo } from '../types';

export interface ImageModificationInfo {
	name:
		| 'addBorder'
		| 'sepia'
		| 'brightness'
		| 'resize'
		| 'annotate'
		| 'crop'
		| 'upscale'
		| 'mirror'
		| 'rotate'
		| 'greyscale';
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	arguments: any;
}

export interface ModificationProps {
	imageInfo: FrontendImageInfo;
	imageInfoAfterModification: FrontendImageInfo;
	onApply: (modification: ImageModificationInfo) => void;
}
