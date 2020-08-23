import { Dimensions } from '../electron/main-interop';

export type RendererCallFunction =
    | 'setOriginalImage'
    | 'setOriginalImageDimensions'
    | 'setModifiedImage'
    | 'setModifiedImageDimensions'
    | 'setModificationInProgress'
    | 'showNotification'
    | 'setColorAtPixel';

export interface RendererCallFunctionRequest {
  func: RendererCallFunction,
  funcArgs: any[];
}

export type ElectronCallFunction =
    | 'openImage'
    | 'applyModifications'
    | 'saveImage'
    | 'setLanguage'
    | 'getColorAtPixel';

export interface ElectronCallFunctionRequest {
  func: ElectronCallFunction;
  funcArgs: any[];
}

export interface FrontendImageInfo {
  base64: string;
  dimensions: Dimensions;
}
