import { createReducer } from '../helpers/create-reducer';

import initialState from './initial-state';
import {
  AppActionTypes, SetModificationInProgressAction, SetModifiedImageAction, SetOriginalImageAction
} from './types';

const Reducer = createReducer(initialState, {
  [AppActionTypes.SET_ORIGINAL_IMAGE]: (state, action: SetOriginalImageAction) => {
    return {
      ...state,
      originalImage: action.image
    };
  },
  [AppActionTypes.SET_MODIFIED_IMAGE]: (state, action: SetModifiedImageAction) => {
    return {
      ...state,
      modifiedImage: action.image
    };
  },
  [AppActionTypes.SET_MODIFICATION_IN_PROGRESS]: (state, action: SetModificationInProgressAction) => {
    return {
      ...state,
      modificationInProgress: action.modificationInProgress
    };
  }
});

export default Reducer;
