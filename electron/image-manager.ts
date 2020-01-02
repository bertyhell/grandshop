/// <reference path="../node_modules/@types/gm/index.d.ts" />
import { ParsedPath } from "path";
import { ImageModificationInfo } from '../src/shared/types';
import { State } from 'gm';

const getUuid = require('uuid/v1');
const isDev = require('electron-is-dev');
const { app } = require('electron');
const promiseUtils = require('blend-promise-utils');

const CustomError = require('./helpers/error').default;

const path = require('path');
const fs = require('fs-extra');
const gm = require('gm').subClass({
  // The lib folder is stored under resources if the app is packaged and under the root folder if the app is running in development
  appPath: path.resolve(__dirname, '..', ...(isDev ? ['lib'] : ['..', 'lib']), 'GraphicsMagick-1.3.33-Q16') + path.sep
});

const tempFolder: string = app.getPath("temp");

const convertExePath = path.resolve(__dirname, '..', ...(isDev ? ['lib'] : ['..', 'lib']), 'GraphicsMagick-1.3.33-Q16') + path.sep + 'convert.exe';
if (!fs.existsSync(convertExePath)) {
  throw 'Can\'t find image modification library at: ' + convertExePath;
}

exports.default = class ImageManager {
  private static originalImagePathParts: ParsedPath;
  private static originalImagePath: string;
  private static originalImageBuffer: Buffer;

  private static modifiedImageBuffer: Buffer;

  private static imageModifications: ImageModificationInfo[] = [];

  public static async updateModifications(modifications: ImageModificationInfo[]): Promise<Buffer> {
    return new Promise<Buffer>(async (resolve, reject) => {
      try {
        this.imageModifications = modifications; // TODO diff modifications and only render new images instead of running all modifications on every change
        let gmState = gm(this.originalImageBuffer, this.originalImagePathParts.name + this.originalImagePathParts.ext);
        for (const modification of this.imageModifications) {
          // try {
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
