import { ImageInfo } from '../types';

export interface ImageModificationInfo {
  name:
      | 'addBorder'
      | 'sepia'
      | 'brightness'
      | 'resize';
  arguments: any;
}

export interface ModificationProps {
  imageInfo: ImageInfo,
  onApply: (modification: ImageModificationInfo) => void,
}
