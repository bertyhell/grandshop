import path, { ParsedPath } from 'path';
import fs from 'fs-extra';
import sharp, { Sharp } from 'sharp';
import bmp from '@vingle/bmp-js';

import { CustomError } from '../error';
import mods, { ImageInfo } from './modifications';
import { ImageModificationInfo } from './types';
import { isNil } from 'lodash';
import { Dimensions, Extension } from '../types';

export default class ImageManager {
	private static originalImagePathParts: ParsedPath;
	private static originalImagePath: string;
	private static originalImageBuffer: Buffer;
	private static originalImageDimensions: Dimensions;

	private static modifiedImageBuffer: Buffer;
	private static modifiedImageDimensions: Dimensions;

	public static updateModifications = (
		modifications: ImageModificationInfo[]
	): Promise<ImageInfo> => {
		// eslint-disable-next-line no-async-promise-executor
		return new Promise(async (resolve, reject) => {
			try {
				ImageManager.modifiedImageBuffer = ImageManager.originalImageBuffer;
				ImageManager.modifiedImageDimensions = await ImageManager.getDimensions(
					ImageManager.modifiedImageBuffer
				);

				// Apply each modification by passing it the image buffer and the args that were defined in the frontend
				for (const modification of modifications) {
					// eslint-disable-next-line @typescript-eslint/no-explicit-any
					const func: ((imageInfo: ImageInfo, ...args: any[]) => Promise<Buffer>) | null =
						mods[modification.name];
					if (!func) {
						throw new CustomError(
							'requested to execute unsupported image modification',
							null,
							{
								modification,
								originalImagePath: ImageManager.originalImagePath,
							}
						);
					}
					ImageManager.modifiedImageBuffer = await func.bind(ImageManager)(
						{
							buffer: ImageManager.modifiedImageBuffer,
							dimensions: ImageManager.modifiedImageDimensions,
						},
						...modification.arguments
					);

					// Since the modifier can change the image size, we need to update it after each modification
					ImageManager.modifiedImageDimensions = await ImageManager.getDimensions(
						ImageManager.modifiedImageBuffer
					);
				}
				resolve({
					buffer: ImageManager.modifiedImageBuffer,
					dimensions: ImageManager.modifiedImageDimensions,
				});
			} catch (err) {
				reject(
					new CustomError('Failed to apply image modifications', err, {
						modifications,
						originalImagePath: ImageManager.originalImagePath,
					})
				);
			}
		});
	};

	public static async setOriginalImagePath(
		absoluteFilePath: string
	): Promise<{ buffer: Buffer; dimensions: Dimensions }> {
		this.originalImagePathParts = path.parse(absoluteFilePath);
		this.originalImagePath = absoluteFilePath;
		const extension = path.parse(absoluteFilePath).ext;

		// Always convert input images to a format that both sharp and graphicmagic understand and is lossless: PNG
		const buffer = await fs.readFile(absoluteFilePath);
		this.originalImageBuffer = await this.bufferToSharp(buffer, extension.substring(1))
			.png()
			.toBuffer();
		this.originalImageDimensions = await this.getDimensions(this.originalImageBuffer);
		return {
			buffer: this.originalImageBuffer,
			dimensions: this.originalImageDimensions,
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

	public static bufferToSharp(
		imageBuffer: Buffer,
		extension: Extension | string = Extension.png
	): Sharp {
		// BMP file types
		// https://github.com/lovell/sharp/issues/806#issuecomment-1220062412
		if (extension.toLowerCase() === Extension.bmp) {
			const bitmap = bmp.decode(imageBuffer, true); // boolean flag to convert to RGBA
			return sharp(bitmap.data, {
				raw: {
					width: bitmap.width,
					height: bitmap.height,
					channels: 4,
				},
			});
		}

		// Supported file types
		if ((Object.values(Extension) as string[]).includes(extension)) {
			return sharp(imageBuffer);
		}

		// Unsupported file types
		throw new CustomError('Failed to open file with unsupported extension', null, {
			extension,
			validExtensions: Object.values(Extension),
		});
	}

	public static async sharpToBuffer(sharp: Sharp): Promise<Buffer> {
		return await sharp.toBuffer();
	}

	private static async getDimensions(buffer: Buffer): Promise<Dimensions> {
		try {
			const metadata = await this.bufferToSharp(buffer).metadata();
			if (!metadata || isNil(metadata.width) || isNil(metadata.height)) {
				throw new CustomError('Failed to fetch dimensions from metadata', null, {
					metadata,
				});
			}
			return {
				width: metadata.width,
				height: metadata.height,
			};
		} catch (err) {
			throw new CustomError('Failed to get dimensions from image', err);
		}
	}
}
