import { Action } from 'redux';

export enum AppActionTypes {
  SET_ORIGINAL_IMAGE = '@@SET_MODIFIED_IMAGE',
  SET_MODIFIED_IMAGE = '@@SET_ORIGINAL_IMAGE',
  SET_MODIFICATION_IN_PROGRESS = '@@SET_MODIFICATION_IN_PROGRESS',
}

export interface SetOriginalImageAction extends Action {
  image: string | null;
}

export interface SetModifiedImageAction extends Action {
  image: string | null;
}

export interface SetModificationInProgressAction extends Action {
  modificationInProgress: boolean;
}

export interface AppState {
  readonly originalImage: string | null;
  readonly modifiedImage: string | null;
  readonly modificationInProgress: boolean;
}
