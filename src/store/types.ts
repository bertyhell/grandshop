import { Action } from 'redux';

import { ImageInfo } from '../shared/types';

export enum AppActionTypes {
  SET_ORIGINAL_IMAGE = '@@SET_MODIFIED_IMAGE',
  SET_MODIFIED_IMAGE = '@@SET_ORIGINAL_IMAGE',
  SET_MODIFICATION_IN_PROGRESS = '@@SET_MODIFICATION_IN_PROGRESS',
}

export interface SetOriginalImageAction extends Action {
  imageInfo: ImageInfo | null;
}

export interface SetModifiedImageAction extends Action {
  imageInfo: ImageInfo | null;
}

export interface SetModificationInProgressAction extends Action {
  modificationInProgress: boolean;
}

export interface AppState {
  readonly originalImage: ImageInfo | null;
  readonly modifiedImage: ImageInfo | null;
  readonly modificationInProgress: boolean;
}
