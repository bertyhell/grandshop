import { FunctionComponent } from 'react';

import AddBorder from './AddBorder/AddBorder';
import Brightness from './Brightness/Brightness';
import Sepia from './Sepia/Sepia';
import Resize from './Resize/Resize';
import Annotate from './Annotate/Annotate';

export interface ModificationComponent {
  component: FunctionComponent<any>, // React component to load for this modification
  label: string, // Label shown in the modification dropdown list
  title: string, // Text show as the title of the section in the controls sidebar
}

const modificationComponents: {[name: string]: ModificationComponent} = {
  addBorder: {
    component: AddBorder,
    label: 'Voeg een gekleurde rand toe',
    title: "Rand {0}",
  },
  brightness: {
    component: Brightness,
    label: 'Pas de helderheid aan',
    title: "Helderheid {0}",
  },
  sepia: {
    component: Sepia,
    label: 'Voeg een sepia effect toe',
    title: "Sepia {0}",
  },
  resize: {
    component: Resize,
    label: 'Afbeelding verkleinen (bestandsgrootte)',
    title: "Verkleinen {0}",
  },
  annotate: {
    component: Annotate,
    label: 'Tekst toevoegen onder de afbeelding',
    title: "Bijschrift {0}",
  },
};

export default modificationComponents;
