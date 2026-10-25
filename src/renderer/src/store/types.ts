import type { UnknownAction } from '@reduxjs/toolkit';

import { FrontendImageInfo } from '../../../shared/types';

export enum AppActionTypes {
	SET_ORIGINAL_IMAGE = '@@SET_ORIGINAL_IMAGE',
	CLEAR_MODIFIED_IMAGES = '@@CLEAR_MODIFIED_IMAGES',
	SET_MODIFIED_IMAGE = '@@SET_MODIFIED_IMAGE',
	SET_MODIFIED_IMAGES = '@@SET_MODIFIED_IMAGES',
	SET_MODIFICATION_IN_PROGRESS = '@@SET_MODIFICATION_IN_PROGRESS',
	SET_COLOR_AT_PIXEL = '@@SET_COLOR_AT_PIXEL',
}

export interface SetOriginalImageAction extends UnknownAction {
	imageInfo: FrontendImageInfo | null;
}

export interface SetModifiedImageAction extends UnknownAction {
	imageInfo: FrontendImageInfo | null;
	index: number;
}

export interface SetModifiedImagesAction extends UnknownAction {
	imageInfos: (FrontendImageInfo | null)[];
}

export interface ClearModifiedImagesAction extends UnknownAction {}

export interface SetModificationInProgressAction extends UnknownAction {
	modificationInProgress: boolean;
}

export interface SetColorAtPixelAction extends UnknownAction {
	color: string;
}

export interface AppState {
	readonly originalImage: FrontendImageInfo | null;
	readonly modifiedImages: (FrontendImageInfo | null)[];
	readonly modificationInProgress: boolean;
	readonly color: string | null;
}
