import { State } from 'gm';

export function brightness(gmState: State, brightness: number): State {
  return (gmState.modulate as any)(brightness);
}
