import { FrontendImageInfo } from '../types';

export interface ImageModificationInfo {
	name:
		| 'addBorder'
		| 'sepia'
		| 'brightness'
		| 'resize'
		| 'annotate'
		| 'crop';
	arguments: any;
}

export interface ModificationProps {
	imageInfo: FrontendImageInfo,
	onApply: (modification: ImageModificationInfo) => void,
}
