import { AppState } from './types';

const initialState: AppState = Object.freeze({
  originalImage: null,
  modifiedImage: null,
  modificationInProgress: false,
});

export default initialState;
