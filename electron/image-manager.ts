/// <reference path="../node_modules/@types/gm/index.d.ts" />
import { ParsedPath } from "path";
import { ImageModificationInfo } from '../src/shared/types';
import { State } from 'gm';
import { Readable } from "stream";
const getUuid = require('uuid/v1');

const CustomError = require('./helpers/error').default;

const path = require('path');
const fs = require('fs-extra');
const gm = require('gm').subClass({
  appPath: path.resolve(__dirname, '..', 'lib', 'GraphicsMagick-1.3.33-Q16') + path.sep,
  imageMagick: true
});

exports.default = class ImageManager {
  private static originalImagePathParts: ParsedPath;
  private static originalImagePath: string;
  private static originalImageBuffer: Buffer;

  private static modifiedImagePath: string;
  private static modifiedImageBuffer: Buffer;

  private static tmpFolder: string = path.resolve(__dirname, '..', 'tmp') + path.sep;

  private static imageModifications: ImageModificationInfo[] = [];

  public static async updateModifications(modifications: ImageModificationInfo[]): Promise<Buffer> {
    try {
      this.imageModifications = modifications; // TODO diff modifications and only render new images instead of running all modifications on every change
      let gmState = gm(this.originalImagePath);
      for (const modification of this.imageModifications) {
        try {
          const func: ((gmState: State, ...args: any[]) => State) | null = this[modification.name];
          if (!func) {
            throw new CustomError('requested to execute unsupported image modification', null, {
              modification,
              originalImagePath: this.originalImagePath
            });
          }
          gmState = await func.bind(ImageManager)(gmState, ...modification.arguments);
        } catch (err) {
          console.error(new CustomError('Failed to execute modification', err, { modification }));
        }
      }
      this.modifiedImagePath = await this.writeGmState(gmState);
      this.modifiedImageBuffer = await fs.readFile(this.modifiedImagePath);
      return this.modifiedImageBuffer;
    } catch (err) {
      throw new CustomError('Failed to apply image modifications', err, { modifications, originalImagePath: this.originalImagePath });
    }
  }

  public static async setOriginalImagePath(absoluteFilePath: string): Promise<Buffer> {
    this.originalImagePathParts = path.parse(absoluteFilePath);
    this.originalImagePath = absoluteFilePath;
    this.originalImageBuffer = await fs.readFile(absoluteFilePath);
    return this.originalImageBuffer;
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

  public static getModifiedImagePath(): string {
    return this.modifiedImagePath;
  }

  private static getNewImagePath(original: string): string {
    const pathParts = path.parse(original);
    return path.resolve(this.tmpFolder, `${pathParts.name}-${getUuid()}${pathParts.ext}`);
  }

  private static writeGmState(gmState: State): Promise<string> {
    return new Promise<string>((resolve, reject) => {
      const modifiedImagePath = this.getNewImagePath(this.originalImagePath);
      gmState.write(modifiedImagePath, async (err: Error | null, stdout: Readable, stderr: Readable, cmd: string) => {
        if (err) {
          reject(new CustomError(
              'Failed to save image to disk',
              err,
              {
                stdout: await this.streamToString(stdout),
                stderr: await this.streamToString(stderr),
                cmd,
              }
              )
          );
        } else {
          resolve(modifiedImagePath);
        }
      });
    });
  }

  private static streamToString(stream: Readable) {
    const chunks: any[] = [];
    return new Promise((resolve, reject) => {
      stream.on('data', chunk => chunks.push(chunk));
      stream.on('error', reject);
      stream.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')))
    })
  }

  private static addBorder(gmState: State, thickness: number, color: string): State {
    return gmState
        .borderColor(color)
        .border(thickness, thickness);
  }

  private static sepia(gmState: State): State {
    return gmState.sepia();
  }

  private static brightness(gmState: State, brightness: number): State {
    return (gmState.modulate as any)(brightness);
  }
};
