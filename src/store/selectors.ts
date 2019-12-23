import { AppState } from './types';

const selectOriginalImage = (state: {app: AppState}) => {
  return state.app.originalImage;
};

const selectModifiedImage = (state: {app: AppState}) => {
  return state.app.modifiedImage;
};

export { selectOriginalImage, selectModifiedImage };
