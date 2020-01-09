import { State } from 'gm';

import { addBorder } from './AddBorder/modification';
import { brightness } from './Brightness/modification';
import { sepia } from './Sepia/modification';
import { resize } from './Resize/modification';
import { annotate } from './Annotate/modification';

export type ModificationFunc = (gmState: State, ...params: any[]) => State;

// List of functions that can modify an image by calling GraphicsMagick functions on its gm state
const modificationFuncs: {[name: string]: ModificationFunc} = {
  addBorder,
  brightness,
  sepia,
  resize,
  annotate,
};

export default modificationFuncs;
