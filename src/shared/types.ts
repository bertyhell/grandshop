import { Dimensions } from 'gm';

export type RendererCallFunction =
    | 'setOriginalImage'
    | 'setOriginalImageDimensions'
    | 'setModifiedImage'
    | 'setModifiedImageDimensions'
    | 'showNotification';

export interface RendererCallFunctionRequest {
  func: RendererCallFunction,
  funcArgs: any[];
}

export type ElectronCallFunction =
    | 'openImage'
    | 'applyModifications'
    | 'saveImage';

export interface ElectronCallFunctionRequest {
  func: ElectronCallFunction;
  funcArgs: any[];
}

export interface ImageInfo {
  base64: string;
  dimensions: Dimensions;

}
