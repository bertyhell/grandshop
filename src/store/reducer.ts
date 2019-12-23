import { createReducer } from '../helpers/create-reducer';

import initialState from './initial-state';
import {
  AppActionTypes, SetModifiedImageAction, SetOriginalImageAction
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
  }
});

export default Reducer;
