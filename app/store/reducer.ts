import { createReducer } from '../helpers/create-reducer';

import initialState from './initial-state';
import {
  AppActionTypes, SetColorAtPixelAction, SetModificationInProgressAction, SetModifiedImageAction, SetOriginalImageAction
} from './types';

const Reducer = createReducer(initialState, {
  [AppActionTypes.SET_ORIGINAL_IMAGE]: (state, action: SetOriginalImageAction) => {
    return {
      ...state,
      originalImage: action.imageInfo
    };
  },
  [AppActionTypes.SET_MODIFIED_IMAGE]: (state, action: SetModifiedImageAction) => {
    return {
      ...state,
      modifiedImage: action.imageInfo
    };
  },
  [AppActionTypes.SET_MODIFICATION_IN_PROGRESS]: (state, action: SetModificationInProgressAction) => {
    return {
      ...state,
      modificationInProgress: action.modificationInProgress
    };
  },
  [AppActionTypes.SET_COLOR_AT_PIXEL]: (state, action: SetColorAtPixelAction) => {
    return {
      ...state,
      color: action.color
    };
  }
});

export default Reducer;
