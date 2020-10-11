export interface ImageBorderInfo {
  path: string;
  name: string;
  x: number;
  y: number;
}

export function analyseBorderPath(borderPath: string): ImageBorderInfo | null {
  /**
   *  Recognise paths of format: c:/test/filename_12_14.png
   *  and extracts:
   *  filename
   *  x coordinate
   *  y coordinate
   **/
  const matchGroups = borderPath.match(/.*[/\\]([^/\\]*)_([0-9]+)_([0-9]+)\..*/);
  if (!matchGroups || matchGroups.length <= 1) {
    return null;
  }
  return {
    path: borderPath,
    name: matchGroups[1],
    x: parseInt(matchGroups[2], 10),
    y: parseInt(matchGroups[3], 10),
  };
}
