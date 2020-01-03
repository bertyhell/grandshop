import { addBorder } from './AddBorder/modification';
import { brightness } from './Brightness/modification';
import { sepia } from './Sepia/modification';
import { State } from 'gm';

export type ModificationFunc = (gmState: State, ...params: any[]) => State;

// List of functions that can modify an image by calling GraphicsMagick functions on its gm state
const modificationFuncs: {[name: string]: ModificationFunc} = {
  addBorder,
  brightness,
  sepia,
};

export default modificationFuncs;
