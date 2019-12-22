/// <reference path="../node_modules/@types/gm/index.d.ts" />
const CustomError = require('./helpers/error').default;

const path = require('path');
const fs = require('fs');
const gm = require('gm').subClass({
  appPath: path.resolve(__dirname, '..', 'lib', 'GraphicsMagick-1.3.33-Q16') + path.sep,
  imageMagick: true
});
const getUuid = require('uuid/v1');

interface ImageModification {
  name: 'addBorder'; // TODO add more modifications
  arguments: any;
}

exports.default = class ImageManager {
  private static originalImagePath: string;
  private static modifiedImagePath: string;
  private static imageModifications: ImageModification[] = [];
  private static tmpFolder: string = path.resolve(__dirname, '..', 'tmp') + path.sep;

  public static async updateModifications(modifications: ImageModification[]): Promise<string> {
    try {
      this.imageModifications = modifications; // TODO diff modifications and only render new images instead of running all modifications on every change
      this.modifiedImagePath = this.originalImagePath;
      for (const modification of this.imageModifications) {
        try {
          const func: ((...args: any[]) => Promise<string>) | null = this[modification.name];
          if (!func) {
            throw new CustomError('requested to execute unsupported image modification', null, {
              modification,
              originalImagePath: this.originalImagePath
            });
          }
          this.modifiedImagePath = await func.bind(ImageManager)(...modification.arguments);
        } catch (err) {
          console.error(new CustomError('Failed to execute modification', err, { modification }));
        }
      }
      return this.modifiedImagePath;
    } catch (err) {
      throw new CustomError('Failed to apply image modifications', err, {modifications, originalImagePath: this.originalImagePath});
    }
  }

  public static setOriginalImagePath(absoluteFilePath: string) {
    this.originalImagePath = this.getNewImagePath(absoluteFilePath);
    fs.copyFileSync(absoluteFilePath, this.originalImagePath);
  }

  public static getModifiedImagePath(): string {
    return this.modifiedImagePath;
  }

  private static getNewImagePath(original: string): string {
    const pathParts = path.parse(original);
    return `${this.tmpFolder}${path.sep}${pathParts.name}-${getUuid()}${pathParts.ext}`;
  }

  private static addBorder(thickness: number, color: string): Promise<string> {
    return new Promise<string>((resolve, reject) => {
      try {
        const modifiedImagePath = this.getNewImagePath(this.originalImagePath);
        gm(this.modifiedImagePath)
            .borderColor(color)
            .border(thickness, thickness)
            .write(modifiedImagePath, (err: Error) => { // TODO see if we can avoid saving and loading the image for every modification (gm streams maybe?)
              if (err) {
                console.error(err);
                reject(err);
              } else {
                resolve(modifiedImagePath);
              }
            });
        return
      } catch (err) {
        console.error(err);
        reject(err);
      }
    });
  }
};
