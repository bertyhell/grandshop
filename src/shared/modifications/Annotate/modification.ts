import { State } from 'gm';

export function annotate(gmState: State, width: number, height: number, fontSize: number, margin: number, lines: string[]): State {
  let state = gmState
      .extent(width, height + margin + lines.length * fontSize + margin)
      .font('helvetica', fontSize)
      .fill("#000");

  lines.forEach((line, index) => {
    state = state
        .drawText(margin, height + margin + fontSize * 0.8 + index * fontSize, line);
  });
  return state;
}
