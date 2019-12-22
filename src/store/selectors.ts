import { AppState } from './types';

const selectOriginalImageUrl = (state: {app: AppState}) => {
  return state.app.originalImageUrl;
};

const selectModifiedImageUrl = (state: {app: AppState}) => {
  return state.app.modifiedImageUrl;
};

export { selectOriginalImageUrl, selectModifiedImageUrl };
