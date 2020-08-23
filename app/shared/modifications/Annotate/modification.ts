import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';
import { Sharp } from 'sharp';
import TextToSVG from 'text-to-svg';
import { clamp } from 'lodash';

export interface SvgMetrics {
  x: number;
  y: number;
  baseline: number;
  width: number;
  height: number;
  ascender: number;
  descender: number;
}

export async function annotate(imageInfo: ImageInfo, text: string, fontSize: number, xPosition: number, yPosition: number): Promise<Buffer> {
  let sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

  const compositeImages = text.split('\n').map((line: string, index: number) => {
    // generate text as svg
    const textToSVG = TextToSVG.loadSync();

    const options = {
      x: 0,
      y: 0,
      fontSize: Math.round(fontSize),
      anchor: 'top',
      attributes: {
        fill: 'black',
        // stroke: 'black'
      }
    };

    const svg = textToSVG.getSVG(line, options);
    const svgMetrics: SvgMetrics = textToSVG.getMetrics(line, options);

    const left = clamp(Math.round(imageInfo.dimensions.width * xPosition / 100 - svgMetrics.width / 2), 0, imageInfo.dimensions.width);
    const top = clamp(Math.round(imageInfo.dimensions.height * yPosition / 100 - svgMetrics.height / 2 + index * svgMetrics.height), 0, imageInfo.dimensions.height);

    return { input: new Buffer(svg), left, top };
  });

  sharp = sharp.composite(compositeImages)

  return ImageManager.sharpToBuffer(sharp);
}
