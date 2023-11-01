import { FunctionComponent } from 'react';

import i18n from '../translations/i18n';

import AddBorder from './AddBorder/AddBorder';
import Brightness from './Brightness/Brightness';
import Sepia from './Sepia/Sepia';
import Resize from './Resize/Resize';
import Annotate from './Annotate/Annotate';
import Crop from './Crop/Crop';
import Upscale from './Upscale/Upscale';
import Rotate from './Rotate/Rotate';
import Mirror from './Mirror/Mirror';
import Greyscale from './Greyscale/Greyscale';

export interface ModificationComponent {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	component: FunctionComponent<any>; // React component to load for this modification
	label: string; // Label shown in the modification dropdown list
	title: string; // Text show as the title of the section in the controls sidebar
}

const getModificationComponents = (): { [name: string]: ModificationComponent } => ({
	addBorder: {
		component: AddBorder,
		label: i18n.t('shared/modifications/components___add-a-colored-border'),
		title: i18n.t('shared/modifications/components___border-0'),
	},
	brightness: {
		component: Brightness,
		label: i18n.t('shared/modifications/components___adjust-the-brightness'),
		title: i18n.t('shared/modifications/components___brightness-0'),
	},
	sepia: {
		component: Sepia,
		label: i18n.t('shared/modifications/components___add-a-sepia-effect'),
		title: i18n.t('shared/modifications/components___sepia-0'),
	},
	resize: {
		component: Resize,
		label: i18n.t('shared/modifications/components___reduce-image-file-size'),
		title: i18n.t('shared/modifications/components___shrink-0'),
	},
	annotate: {
		component: Annotate,
		label: i18n.t('shared/modifications/components___add-text'),
		title: i18n.t('shared/modifications/components___caption-0'),
	},
	crop: {
		component: Crop,
		label: i18n.t('shared/modifications/components___crop-image'),
		title: i18n.t('shared/modifications/components___crop-0'),
	},
	upscale: {
		component: Upscale,
		label: i18n.t('shared/modifications/components___upscale-image'),
		title: i18n.t('shared/modifications/components___upscale-0'),
	},
	rotate: {
		component: Rotate,
		label: i18n.t('shared/modifications/components___rotate-image'),
		title: i18n.t('shared/modifications/components___rotate-0'),
	},
	mirror: {
		component: Mirror,
		label: i18n.t('shared/modifications/components___mirror-image'),
		title: i18n.t('shared/modifications/components___mirror-0'),
	},
	greyscale: {
		component: Greyscale,
		label: i18n.t('shared/modifications/components___greyscale-image'),
		title: i18n.t('shared/modifications/components___greyscale-0'),
	},
});

export default getModificationComponents;
