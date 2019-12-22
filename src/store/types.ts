import { Action } from 'redux';

export enum AppActionTypes {
  SET_ORIGINAL_IMAGE_URL = '@@SET_MODIFIED_IMAGE_URL',
  SET_MODIFIED_IMAGE_URL = '@@SET_ORIGINAL_IMAGE_URL',
}

export interface SetOriginalImageUrlAction extends Action {
  url: string | null;
}

export interface SetModifiedImageUrlAction extends Action {
  url: string | null;
}

export interface AppState {
  readonly originalImageUrl: string | null;
  readonly modifiedImageUrl: string | null;
}
