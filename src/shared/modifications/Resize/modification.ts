import { State } from 'gm';

export function resize(gmState: State, size: number): State {
  return gmState.resize(size, size);
}
