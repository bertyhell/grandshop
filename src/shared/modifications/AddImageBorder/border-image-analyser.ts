export interface ImageBorderInfo {
  path: string;
  name: string;
  cornerX: number;
  cornerY: number;
  edgeX: number;
  edgeY: number;
}

export function analyseBorderPath(borderPath: string): ImageBorderInfo | null {
  /**
   *  Recognise paths of format: c:/test/filename_12_14_105_82.png
   *  and extracts:
   *  filename
   *  corner x coordinate
   *  corner y coordinate
   *  edge x coordinate
   *  edge y coordinate
   *
   *             corner x     edge x
   *                   ⊢-⊣ ⊢-----⊣
   *
   *             T     +-+-+-----+-+-----+-+-----+-+-----+-+-----+-+-+
   *   corner y  |     +-+  +---+   +---+   +---+   +---+   +---+  +-+
   *             ⊥     |                                             |
   *             T     +-+                                         +-+
   *             |     |  +                                       +  |
   *     edge y  |     |  +                                       +  |
   *             |     +-+                                         +-+
   *             ⊥     |                                             |
   *                   +-+                                         +-+
   *                   |  +                                       +  |
   *                   |  +                                       +  |
   *                   +-+                                         +-+
   *                   |                                             |
   *                   +-+  +---+   +---+   +---+   +---+   +---+  +-+
   *                   +-+-+-----+-+-----+-+-----+-+-----+-+-----+-+-+
   **/
  const matchGroups = borderPath.match(/.*[/\\]([^/\\]*)_([0-9]+)_([0-9]+)_([0-9]+)_([0-9]+)\..*/);
  if (!matchGroups || matchGroups.length <= 1) {
    return null;
  }
  return {
    path: borderPath,
    name: matchGroups[1],
    cornerX: parseInt(matchGroups[2], 10),
    cornerY: parseInt(matchGroups[3], 10),
    edgeX: parseInt(matchGroups[4], 10),
    edgeY: parseInt(matchGroups[5], 10),
  };
}
