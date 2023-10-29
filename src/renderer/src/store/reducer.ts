import { createReducer, ImageAction } from '../helpers/create-reducer';

import initialState from './initial-state';
import {
	AppActionTypes,
	AppState,
	SetColorAtPixelAction,
	SetModificationInProgressAction,
	SetModifiedImageAction,
	SetOriginalImageAction,
} from './types';

const Reducer = createReducer<AppState>(initialState, {
	[AppActionTypes.SET_ORIGINAL_IMAGE]: (state, action: ImageAction) => {
		return {
			...state,
			originalImage: (action as SetOriginalImageAction).imageInfo,
		};
	},
	[AppActionTypes.SET_MODIFIED_IMAGE]: (state, action: ImageAction) => {
		return {
			...state,
			modifiedImage: (action as SetModifiedImageAction).imageInfo,
		};
	},
	[AppActionTypes.SET_MODIFICATION_IN_PROGRESS]: (state, action: ImageAction) => {
		return {
			...state,
			modificationInProgress: (action as SetModificationInProgressAction)
				.modificationInProgress,
		};
	},
	[AppActionTypes.SET_COLOR_AT_PIXEL]: (state, action: ImageAction) => {
		return {
			...state,
			color: (action as SetColorAtPixelAction).color,
		};
	},
});

export default Reducer;
