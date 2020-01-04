import {
  AppActionTypes,
  SetOriginalImageAction,
  SetModifiedImageAction, SetModificationInProgressAction
} from './types';
import { ImageInfo } from '../shared/types';

export const setOriginalImageAction = (imageInfo: ImageInfo | null): SetOriginalImageAction => ({
  imageInfo,
  type: AppActionTypes.SET_ORIGINAL_IMAGE
});

export const setModifiedImageAction = (imageInfo: ImageInfo | null): SetModifiedImageAction => ({
  imageInfo,
  type: AppActionTypes.SET_MODIFIED_IMAGE
});

export const setModificationInProgressAction = (modificationInProgress: boolean): SetModificationInProgressAction => {
  return {
    modificationInProgress,
    type: AppActionTypes.SET_MODIFICATION_IN_PROGRESS
  };
};
