import {
  AppActionTypes,
  SetOriginalImageAction,
  SetModifiedImageAction, SetModificationInProgressAction
} from './types';

export const setOriginalImageAction = (image: string | null): SetOriginalImageAction => ({
  image,
  type: AppActionTypes.SET_ORIGINAL_IMAGE
});

export const setModifiedImageAction = (image: string | null): SetModifiedImageAction => ({
  image,
  type: AppActionTypes.SET_MODIFIED_IMAGE
});

export const setModificationInProgressAction = (modificationInProgress: boolean): SetModificationInProgressAction => {
  console.log('set modificationInProgress action: ', modificationInProgress);
  return {
    modificationInProgress,
    type: AppActionTypes.SET_MODIFICATION_IN_PROGRESS
  };
};
