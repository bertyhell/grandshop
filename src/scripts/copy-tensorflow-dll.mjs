/**
 * On windows, @tensorflow/tfjs-node builds its binding into lib/napi-v8,
 * but copies tensorflow.dll into the folder of the napi version of the current node version (eg: lib/napi-v9).
 * This copies the dll next to every binding, so the binding can be loaded.
 */
import fs from 'fs';
import path from 'path';

if (process.platform === 'win32') {
	const tfjsNodePath = path.resolve(
		import.meta.dirname,
		'../../node_modules/@tensorflow/tfjs-node'
	);
	const dllPath = path.join(tfjsNodePath, 'deps/lib/tensorflow.dll');
	const libPath = path.join(tfjsNodePath, 'lib');

	if (fs.existsSync(dllPath) && fs.existsSync(libPath)) {
		for (const napiFolder of fs.readdirSync(libPath)) {
			const bindingFolder = path.join(libPath, napiFolder);
			if (
				fs.existsSync(path.join(bindingFolder, 'tfjs_binding.node')) &&
				!fs.existsSync(path.join(bindingFolder, 'tensorflow.dll'))
			) {
				fs.copyFileSync(dllPath, path.join(bindingFolder, 'tensorflow.dll'));
				console.log(`Copied tensorflow.dll to ${bindingFolder}`);
			}
		}
	}
}
