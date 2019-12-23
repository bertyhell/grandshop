export type RendererCallFunction = 'setOriginalImage' | 'setModifiedImage' | 'showNotification';
export interface RendererCallFunctionRequest {
  func: RendererCallFunction,
  funcArgs: any[];
}

export type ElectronCallFunction =  'openImage' | 'applyModifications' | 'saveImage';
export interface ElectronCallFunctionRequest {
  func: ElectronCallFunction;
  funcArgs: any[];
}

export interface ImageModificationInfo {
  name: 'addBorder' | 'sepia' | 'brightness';
  arguments: any;
}
