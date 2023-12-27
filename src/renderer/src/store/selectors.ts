import { AppState } from './types';

const selectOriginalImage = (state: { app: AppState }) => {
	return state.app.originalImage;
};

const selectModifiedImages = (state: { app: AppState }) => {
	return state.app.modifiedImages;
};

const selectModifiedImage = (state: { app: AppState }) => {
	return state.app.modifiedImages.at(-1);
};

const selectModificationInProgress = (state: { app: AppState }) => {
	return state.app.modificationInProgress;
};

export {
	selectOriginalImage,
	selectModifiedImage,
	selectModifiedImages,
	selectModificationInProgress,
};
