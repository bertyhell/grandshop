import { Action } from 'redux';

import { FrontendImageInfo } from '../../../shared/types';

export enum AppActionTypes {
	SET_ORIGINAL_IMAGE = '@@SET_MODIFIED_IMAGE',
	SET_MODIFIED_IMAGE = '@@SET_ORIGINAL_IMAGE',
	SET_MODIFICATION_IN_PROGRESS = '@@SET_MODIFICATION_IN_PROGRESS',
	SET_COLOR_AT_PIXEL = '@@SET_COLOR_AT_PIXEL',
}

export interface SetOriginalImageAction extends Action {
	imageInfo: FrontendImageInfo | null;
}

export interface SetModifiedImageAction extends Action {
	imageInfo: FrontendImageInfo | null;
}

export interface SetModificationInProgressAction extends Action {
	modificationInProgress: boolean;
}

export interface SetColorAtPixelAction extends Action {
	color: string;
}

export interface AppState {
	readonly originalImage: FrontendImageInfo | null;
	readonly modifiedImage: FrontendImageInfo | null;
	readonly modificationInProgress: boolean;
	readonly color: string | null;
}
