import {
  AppActionTypes,
  SetOriginalImageAction,
  SetModifiedImageAction
} from './types';

export const setOriginalImageAction = (image: string | null): SetOriginalImageAction => ({
  image,
  type: AppActionTypes.SET_ORIGINAL_IMAGE
});


export const setModifiedImageAction = (image: string | null): SetModifiedImageAction => ({
  image,
  type: AppActionTypes.SET_MODIFIED_IMAGE
});
