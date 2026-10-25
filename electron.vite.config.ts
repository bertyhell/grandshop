import { resolve } from 'path';
import { defineConfig } from 'electron-vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
	main: {},
	preload: {
		build: {
			// Bundle the preload dependencies, so the preload script can run in a sandboxed renderer
			externalizeDeps: false,
		},
	},
	renderer: {
		resolve: {
			alias: {
				'@renderer': resolve('src/renderer/src'),
			},
		},
		plugins: [react()],
		css: {
			preprocessorOptions: {
				scss: {
					// The vendored font awesome scss files still use @import and global functions
					silenceDeprecations: ['import', 'global-builtin'],
				},
			},
		},
		server: {
			port: 3000,
		},
	},
});
