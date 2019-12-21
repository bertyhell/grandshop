// Modules to control application life and create native browser window
const { app, BrowserWindow } = require('electron');
// import { app, BrowserWindow, dialog } from 'electron';
const path = require('path');
const isDev = require('electron-is-dev');
const im = require('imagemagick');

// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow: any;

function createWindow() {
	// Create the browser window.
	mainWindow = new BrowserWindow({
		width: 800,
		height: 600,
		webPreferences: {
			preload: path.join(__dirname, 'preload.js'),
			nodeIntegration: true,
		},
	});

	mainWindow.webContents.openDevTools();

	mainWindow.loadURL(
		isDev ? 'http://localhost:3000' : `file://${path.join(__dirname, '../build/index.html')}`
	);
	// // and load the index.html of the app.
	// mainWindow.loadFile('public/index.html');

	// Open the DevTools.
	// mainWindow.webContents.openDevTools()

	// Emitted when the window is closed.
	mainWindow.on('closed', () => {
		// Dereference the window object, usually you would store windows
		// in an array if your app supports multi windows, this is the time
		// when you should delete the corresponding element.
		mainWindow = null;
	});
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow);

// Quit when all windows are closed.
app.on('window-all-closed', () => {
	// On macOS it is common for applications and their menu bar
	// to stay active until the user quits explicitly with Cmd + Q
	if (process.platform !== 'darwin') {
		app.quit();
	}
});

app.on('activate', () => {
	// On macOS it's common to re-create a window in the app when the
	// dock icon is clicked and there are no other windows open.
	if (mainWindow === null) {
		createWindow();
	}
});

exports.openFileDialog = () => {
	// dialog.showOpenDialogSync(mainWindow);
};

exports.transformImage = (thickness: number, color: string) => {
	im.convert(['../tmp/old.jpg', '-resize', '25x120', '../tmp/new.jpg'], (err: Error, stdout: any) => {
		console.error(err);
		console.log(stdout);
	});
};
