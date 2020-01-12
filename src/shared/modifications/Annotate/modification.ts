import { State } from 'gm';
import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function annotate(imageInfo: ImageInfo, fontSize: number, margin: number, lines: string[]): Promise<Buffer> {
  const gmState: State = ImageManager.bufferToGm(imageInfo.buffer);

  let state = gmState
      .extent(imageInfo.dimensions.width, imageInfo.dimensions.height + margin + lines.length * fontSize + margin)
      .font('helvetica', fontSize)
      .fill("#000");

  lines.forEach((line, index) => {
    state = state
        .drawText(margin, imageInfo.dimensions.height + margin + fontSize * 0.8 + index * fontSize, line);
  });

  return ImageManager.gmToBuffer(state);
}
