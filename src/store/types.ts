import { Action } from 'redux';

import { FrontendImageInfo } from '../shared/types';

export enum AppActionTypes {
  SET_ORIGINAL_IMAGE = '@@SET_MODIFIED_IMAGE',
  SET_MODIFIED_IMAGE = '@@SET_ORIGINAL_IMAGE',
  SET_MODIFICATION_IN_PROGRESS = '@@SET_MODIFICATION_IN_PROGRESS',
}

export interface SetOriginalImageAction extends Action {
  imageInfo: FrontendImageInfo | null;
}

export interface SetModifiedImageAction extends Action {
  imageInfo: FrontendImageInfo | null;
}

export interface SetModificationInProgressAction extends Action {
  modificationInProgress: boolean;
}

export interface AppState {
  readonly originalImage: FrontendImageInfo | null;
  readonly modifiedImage: FrontendImageInfo | null;
  readonly modificationInProgress: boolean;
}
