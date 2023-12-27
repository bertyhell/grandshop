import { AppState } from './types';

const initialState: AppState = Object.freeze({
	originalImage: null,
	modifiedImages: [],
	modificationInProgress: false,
	color: null,
});

export default initialState;
