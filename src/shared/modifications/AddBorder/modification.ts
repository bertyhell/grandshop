import { State } from 'gm';

export function addBorder(gmState: State, thickness: number, color: string): State {
  return gmState
      .borderColor(color)
      .border(thickness, thickness);
}
