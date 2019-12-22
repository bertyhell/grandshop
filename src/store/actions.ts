import {
  AppActionTypes,
  SetOriginalImageUrlAction,
  SetModifiedImageUrlAction
} from './types';

export const setOriginalImageUrlAction = (url: string): SetOriginalImageUrlAction => ({
  url,
  type: AppActionTypes.SET_ORIGINAL_IMAGE_URL
});


export const setModifiedImageUrlAction = (url: string): SetModifiedImageUrlAction => ({
  url,
  type: AppActionTypes.SET_MODIFIED_IMAGE_URL
});
