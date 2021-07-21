import { addImageBorder } from './modification';
import * as path from 'path';
import { loadImage } from '../../helpers/load-image-and-dimensions';
import * as fs from 'fs-extra';

const FIXTURE_IMAGE: string = path.resolve('src/fixtures/paint.jpg');
const BORDER_IMAGE_FLORAL: string = path.resolve('resources/modifications/add-image-border/borders/floral_curly_188_188_314_314.png');

describe('AddImageBorder', () => {
	it('Should add an image border', async () => {
		const imageBuffer = await addImageBorder(await loadImage(FIXTURE_IMAGE), BORDER_IMAGE_FLORAL, 60);
		await fs.writeFile(path.resolve('./test.png'), imageBuffer);
	});
});
