import { Dimensions } from 'gm';

export type RendererCallFunction =
    | 'setOriginalImage'
    | 'setOriginalImageDimensions'
    | 'setModifiedImage'
    | 'setModifiedImageDimensions'
    | 'setModificationInProgress'
    | 'showNotification';

export interface RendererCallFunctionRequest {
  func: RendererCallFunction,
  funcArgs: any[];
}

export type ElectronCallFunction =
    | 'openImage'
    | 'applyModifications'
    | 'saveImage'
    | 'setLanguage';

export interface ElectronCallFunctionRequest {
  func: ElectronCallFunction;
  funcArgs: any[];
}

export interface FrontendImageInfo {
  base64: string;
  dimensions: Dimensions;
}
