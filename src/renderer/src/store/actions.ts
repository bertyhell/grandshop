import {
	AppActionTypes,
	SetOriginalImageAction,
	SetModifiedImageAction,
	SetModificationInProgressAction,
	SetColorAtPixelAction,
	ClearModifiedImagesAction,
	SetModifiedImagesAction,
} from './types';
import { FrontendImageInfo } from '../../../shared/types';

export const setOriginalImageAction = (
	imageInfo: FrontendImageInfo | null
): SetOriginalImageAction => ({
	imageInfo,
	type: AppActionTypes.SET_ORIGINAL_IMAGE,
});

export const setModifiedImageAction = (
	imageInfo: FrontendImageInfo | null,
	index: number
): SetModifiedImageAction => ({
	imageInfo,
	index,
	type: AppActionTypes.SET_MODIFIED_IMAGE,
});

export const setModifiedImagesAction = (
	imageInfos: (FrontendImageInfo | null)[]
): SetModifiedImagesAction => ({
	imageInfos,
	type: AppActionTypes.SET_MODIFIED_IMAGES,
});

export const clearModifiedImagesAction = (): ClearModifiedImagesAction => ({
	type: AppActionTypes.CLEAR_MODIFIED_IMAGES,
});

export const setModificationInProgressAction = (
	modificationInProgress: boolean
): SetModificationInProgressAction => {
	return {
		modificationInProgress,
		type: AppActionTypes.SET_MODIFICATION_IN_PROGRESS,
	};
};

export const setColorAtPixelAction = (color: string): SetColorAtPixelAction => {
	return {
		color,
		type: AppActionTypes.SET_COLOR_AT_PIXEL,
	};
};
