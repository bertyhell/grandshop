const { remote } = (window as any).require('electron');

const mainProcess = remote.require('./main.js');

export function openFileDialog() {
  // mainProcess.openFileDialog();

}

export function transformImage(thickness: number, color: string) {
  mainProcess.transformImage(thickness, color);
}
