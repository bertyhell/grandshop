import { AppState } from './types';

const selectOriginalImage = (state: {app: AppState}) => {
  return state.app.originalImage;
};

const selectModifiedImage = (state: {app: AppState}) => {
  return state.app.modifiedImage;
};

const selectModificationInProgress = (state: {app: AppState}) => {
  return state.app.modificationInProgress;
};

const selectColorAtPixel = (state: {app: AppState}) => {
  return state.app.color;
};

export { selectOriginalImage, selectModifiedImage, selectModificationInProgress, selectColorAtPixel };
