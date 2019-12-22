import { createReducer } from '../helpers/create-reducer';

import initialState from './initial-state';
import {
  AppActionTypes, SetModifiedImageUrlAction, SetOriginalImageUrlAction
} from './types';

const Reducer = createReducer(initialState, {
  [AppActionTypes.SET_ORIGINAL_IMAGE_URL]: (state, action: SetOriginalImageUrlAction) => {
    return {
      ...state,
      originalImageUrl: action.url
    };
  },
  [AppActionTypes.SET_MODIFIED_IMAGE_URL]: (state, action: SetModifiedImageUrlAction) => {
    return {
      ...state,
      modifiedImageUrl: action.url
    };
  }
});

export default Reducer;
