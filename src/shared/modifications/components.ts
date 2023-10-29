import { FunctionComponent } from 'react';

import i18n from '../translations/i18n';

import AddBorder from './AddBorder/AddBorder';
import Brightness from './Brightness/Brightness';
import Sepia from './Sepia/Sepia';
import Resize from './Resize/Resize';
import Annotate from './Annotate/Annotate';
import Crop from './Crop/Crop';

export interface ModificationComponent {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	component: FunctionComponent<any>; // React component to load for this modification
	label: string; // Label shown in the modification dropdown list
	title: string; // Text show as the title of the section in the controls sidebar
}

const modificationComponents: { [name: string]: ModificationComponent } = {
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
};

export default modificationComponents;
