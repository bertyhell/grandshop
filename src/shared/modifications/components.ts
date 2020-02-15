import { FunctionComponent } from 'react';

import i18n from '../translations/i18n';

import AddBorder from './AddBorder/AddBorder';
import Brightness from './Brightness/Brightness';
import Sepia from './Sepia/Sepia';
import Resize from './Resize/Resize';
import Annotate from './Annotate/Annotate';
import Crop from './Crop/Crop';

export interface ModificationComponent {
  component: FunctionComponent<any>, // React component to load for this modification
  label: string, // Label shown in the modification dropdown list
  title: string, // Text show as the title of the section in the controls sidebar
}

const modificationComponents: {[name: string]: ModificationComponent} = {
  addBorder: {
    component: AddBorder,
    label: i18n.t('shared/modifications/components___voeg-een-gekleurde-rand-toe'),
    title: i18n.t('shared/modifications/components___rand-0'),
  },
  brightness: {
    component: Brightness,
    label: i18n.t('shared/modifications/components___pas-de-helderheid-aan'),
    title: i18n.t('shared/modifications/components___helderheid-0'),
  },
  sepia: {
    component: Sepia,
    label: i18n.t('shared/modifications/components___voeg-een-sepia-effect-toe'),
    title: i18n.t('shared/modifications/components___sepia-0'),
  },
  resize: {
    component: Resize,
    label: i18n.t('shared/modifications/components___afbeelding-verkleinen-bestandsgrootte'),
    title: i18n.t('shared/modifications/components___verkleinen-0'),
  },
  annotate: {
    component: Annotate,
    label: i18n.t('shared/modifications/components___tekst-toevoegen-onder-de-afbeelding'),
    title: i18n.t('shared/modifications/components___bijschrift-0'),
  },
  crop: {
    component: Crop,
    label: i18n.t('shared/modifications/components___afbeelding-bijsnijden'),
    title: i18n.t('shared/modifications/components___bijsnijden-0'),
  },
};

export default modificationComponents;
