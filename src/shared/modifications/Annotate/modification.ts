import ImageManager from '../image-manager';
import type { ImageInfo } from '../modifications';
import type { Sharp } from 'sharp';
import TextToSVG from 'text-to-svg';
import { maxBy } from 'lodash';

// generate text as svg
const textToSVG = TextToSVG.loadSync();

interface CompositeImage {
	input: Buffer;
	top: number;
	left: number;
	width: number;
	height: number;
}

export interface SvgMetrics {
	x: number;
	y: number;
	baseline: number;
	width: number;
	height: number;
	ascender: number;
	descender: number;
}

function getMaxTextLocation(compositeImages: CompositeImage[]): { x: number; y: number } {
	const maxHorizontal = maxBy(
		compositeImages,
		(compositeImage) => compositeImage.left + compositeImage.width
	) as CompositeImage;
	const maxVertical = maxBy(
		compositeImages,
		(compositeImage) => compositeImage.top + compositeImage.height
	) as CompositeImage;
	return {
		x: Math.ceil(maxHorizontal.left + maxHorizontal.width),
		y: Math.ceil(maxVertical.top + maxVertical.height),
	};
}

export async function annotate(
	imageInfo: ImageInfo,
	text: string,
	fontSize: number,
	xPosition: number,
	yPosition: number,
	textColor: string,
	backgroundColor: string
): Promise<Buffer> {
	let sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const compositeImages: CompositeImage[] = text
		.split('\n')
		.map((line: string, index: number) => {
			const textLine = line || ' ';
			const options = {
				x: 0,
				y: 0,
				fontSize: Math.round(fontSize),
				anchor: 'left top',
				attributes: {
					fill: textColor,
					// stroke: 'black'
				},
			};

			const svgMetrics: SvgMetrics = textToSVG.getMetrics(textLine, options);
			const svg = textToSVG
				.getSVG(textLine, options)
				.replace(
					'xmlns:xlink="',
					`viewBox="0 0 ${svgMetrics.width} ${svgMetrics.height}" xmlns:xlink="`
				);

			let left = xPosition;
			let top = yPosition + index * svgMetrics.height;

			// Fix strange offset issue when text is placed left or above the existing image
			if (left < 0) {
				left -= svgMetrics.width * (left / svgMetrics.width);
			}
			if (top < 0) {
				top -= svgMetrics.height * (top / svgMetrics.height);
			}

			return {
				input: new Buffer(svg),
				left: Math.round(left),
				top: Math.round(top),
				width: svgMetrics.width,
				height: svgMetrics.height,
			};
		});

	const imageSize = await sharp.metadata();
	const imageWidth = imageSize.width as number;
	const imageHeight = imageSize.height as number;

	const maxTextLocation = getMaxTextLocation(compositeImages);

	sharp = sharp
		// Extend the image so all the text fits
		.extend({
			left: Math.abs(Math.min(xPosition, 0)),
			top: Math.abs(Math.min(yPosition, 0)),
			right: Math.max(maxTextLocation.x, imageWidth) - imageWidth,
			bottom: Math.max(maxTextLocation.y, imageHeight) - imageHeight,
			background: backgroundColor,
		})
		// Apply the text images on the image
		.composite(compositeImages);

	return ImageManager.sharpToBuffer(sharp);
}
