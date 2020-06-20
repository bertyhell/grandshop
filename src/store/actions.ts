import {
  AppActionTypes,
  SetOriginalImageAction,
  SetModifiedImageAction, SetModificationInProgressAction, SetColorAtPixelAction
} from './types';
import { FrontendImageInfo } from '../shared/types';

export const setOriginalImageAction = (imageInfo: FrontendImageInfo | null): SetOriginalImageAction => ({
  imageInfo,
  type: AppActionTypes.SET_ORIGINAL_IMAGE
});

export const setModifiedImageAction = (imageInfo: FrontendImageInfo | null): SetModifiedImageAction => ({
  imageInfo,
  type: AppActionTypes.SET_MODIFIED_IMAGE
});

export const setModificationInProgressAction = (modificationInProgress: boolean): SetModificationInProgressAction => {
  return {
    modificationInProgress,
    type: AppActionTypes.SET_MODIFICATION_IN_PROGRESS
  };
};

export const setColorAtPixelAction = (color: string): SetColorAtPixelAction => {
  return {
    color,
    type: AppActionTypes.SET_COLOR_AT_PIXEL
  };
};
