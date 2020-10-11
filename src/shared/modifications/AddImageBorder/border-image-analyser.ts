export interface ImageBorderInfo {
  path: string;
  name: string;
  x: number;
  y: number;
}

export function analyseBorderPath(borderPath: string): ImageBorderInfo | null {
  const matchGroups = borderPath.match(/^(.*)_([0-9]+)_([0-9]+)\..*$/g);
  console.log('match groups: ', matchGroups);
  if (!matchGroups) {
    return null;
  }
  return {
    path: borderPath,
    name: matchGroups[1],
    x: parseInt(matchGroups[2]),
    y: parseInt(matchGroups[3]),
  };
}
