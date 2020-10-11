import path, { ParsedPath } from "path";
import fs from 'fs-extra';
import sharp, { Sharp } from 'sharp';

import { CustomError } from '../shared/error';
import mods, { ImageInfo } from '../shared/modifications/modifications';
import { ImageModificationInfo } from '../shared/modifications/types';
import { Dimensions } from './main-interop';
import { getImageDimensions } from '../shared/helpers/get-image-dimensions';

export default class ImageManager {
  private static originalImagePathParts: ParsedPath;
  private static originalImagePath: string;
  private static originalImageBuffer: Buffer;
  private static originalImageDimensions: Dimensions;

  private static modifiedImageBuffer: Buffer;
  private static modifiedImageDimensions: Dimensions;

  public static updateModifications = (modifications: ImageModificationInfo[]): Promise<ImageInfo> => {
    return new Promise(async (resolve, reject) => {
      try {
        ImageManager.modifiedImageBuffer = ImageManager.originalImageBuffer;
        ImageManager.modifiedImageDimensions = await ImageManager.getDimensions(ImageManager.modifiedImageBuffer);

        // Apply each modification by passing it the image buffer and the args that were defined in the frontend
        for (const modification of modifications) {
          try {
            const func: ((imageInfo: ImageInfo, ...args: any[]) => Promise<Buffer>) | null = mods[modification.name];
            if (!func) {
              throw new CustomError('requested to execute unsupported image modification', null, {
                modification,
                originalImagePath: ImageManager.originalImagePath
              });
            }
            ImageManager.modifiedImageBuffer = await func.bind(ImageManager)({
              buffer: ImageManager.modifiedImageBuffer,
              dimensions: ImageManager.modifiedImageDimensions
            }, ...modification.arguments);

            // Since the modifier can change the image size, we need to update it after each modification
            ImageManager.modifiedImageDimensions = await ImageManager.getDimensions(ImageManager.modifiedImageBuffer);
          } catch (err) {
            console.error(new CustomError(
              'Failed to apply image modification',
              err,
              { modification, originalImagePath: ImageManager.originalImagePath }
            ));
          }
        }
        resolve({
          buffer: ImageManager.modifiedImageBuffer,
          dimensions: ImageManager.modifiedImageDimensions
        });
      } catch (err) {
        reject(
            new CustomError(
                'Failed to apply image modifications',
                err,
                { modifications, originalImagePath: ImageManager.originalImagePath }
            )
        );
      }
    });
  };

  public static async setOriginalImagePath(absoluteFilePath: string): Promise<{ buffer: Buffer, dimensions: Dimensions }> {
    this.originalImagePathParts = path.parse(absoluteFilePath);
    this.originalImagePath = absoluteFilePath;
    // Always convert input images to a format that both sharp and graphicmagic understand and is lossless: PNG
    this.originalImageBuffer = await this.bufferToSharp(await fs.readFile(absoluteFilePath)).png().toBuffer();
    this.originalImageDimensions = await this.getDimensions(this.originalImageBuffer);
    return {
      buffer: this.originalImageBuffer,
      dimensions: this.originalImageDimensions
    };
  }

  public static getOriginalImagePath(): string {
    return this.originalImagePath;
  }

  public static getOriginalImagePathParts(): ParsedPath {
    return this.originalImagePathParts;
  }

  public static getModifiedImageBuffer(): Buffer {
    return this.modifiedImageBuffer;
  }

  public static bufferToSharp(imageBuffer: Buffer): Sharp {
    return sharp(imageBuffer);
  }

  public static async sharpToBuffer(sharp: Sharp): Promise<Buffer> {
    return await sharp.toBuffer();
  }

  private static async getDimensions(buffer: Buffer): Promise<Dimensions> {
    return getImageDimensions(this.bufferToSharp(buffer));
  }
};
