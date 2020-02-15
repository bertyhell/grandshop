import i18n from '../../translations/i18n';
import { FunctionComponent } from 'react';

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
    label: i18n.t('Voeg een gekleurde rand toe'),
    title: i18n.t('Rand {0}'),
  },
  brightness: {
    component: Brightness,
    label: i18n.t('Pas de helderheid aan'),
    title: i18n.t('Helderheid {0}'),
  },
  sepia: {
    component: Sepia,
    label: i18n.t('Voeg een sepia effect toe'),
    title: i18n.t('Sepia {0}'),
  },
  resize: {
    component: Resize,
    label: i18n.t('Afbeelding verkleinen (bestandsgrootte)'),
    title: i18n.t('Verkleinen {0}'),
  },
  annotate: {
    component: Annotate,
    label: i18n.t('Tekst toevoegen onder de afbeelding'),
    title: i18n.t('Bijschrift {0}'),
  },
  crop: {
    component: Crop,
    label: i18n.t('Afbeelding bijsnijden'),
    title: i18n.t('Bijsnijden {0}'),
  },
};

export default modificationComponents;
