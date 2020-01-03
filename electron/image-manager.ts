/// <reference path="../node_modules/@types/gm/index.d.ts" />
import { ParsedPath } from "path";
import { ImageModificationInfo } from '../src/shared/types';
import { State } from 'gm';
import CustomError from './helpers/error';

import path from 'path';
import fs from 'fs-extra';
import gm from 'gm';

export default class ImageManager {
  private static originalImagePathParts: ParsedPath;
  private static originalImagePath: string;
  private static originalImageBuffer: Buffer;

  private static modifiedImageBuffer: Buffer;

  public static updateModifications(modifications: ImageModificationInfo[]): Promise<Buffer> {
    return new Promise<Buffer>(async (resolve, reject) => {
      try {
        let gmState = gm(this.originalImageBuffer, this.originalImagePathParts.name + this.originalImagePathParts.ext);
        for (const modification of modifications) {
          const func: ((gmState: State, ...args: any[]) => State) | null = this[modification.name];
          if (!func) {
            throw new CustomError('requested to execute unsupported image modification', null, {
              modification,
              originalImagePath: this.originalImagePath
            });
          }
          gmState = func.bind(ImageManager)(gmState, ...modification.arguments);
        }
        this.modifiedImageBuffer = await this.writeGmState(gmState);
        resolve(this.modifiedImageBuffer);
      } catch (err) {
        reject(new CustomError('Failed to apply image modifications', err, { modifications, originalImagePath: this.originalImagePath }));
      }
    });
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

  private static writeGmState(gmState: State): Promise<Buffer> {
    return new Promise<Buffer>((resolve, reject) => {
      try {
        gmState.toBuffer(function(err: any, newBuffer: Buffer) {
          if (err) {
            reject(new CustomError('Failed to write modified image to buffer', err));
          } else {
            resolve(newBuffer);
          }
        });
      } catch (err) {
        reject(new CustomError(
            'Failed to write image to disk',
            err));
      }
    });
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
