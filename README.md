# Grandshop

A image editing program that is really easy to use.

Current features:

- Upscale images (4x)
  ![screenshot of upscaled image](screenshots/07-upscale.jpg)
- Add colored borders
  ![screenshot of colored border](screenshots/01-border.jpg)
- Change the brightness
  ![screenshot of brightness](screenshots/02-brightness.jpg)
- Add a sepia effect
  ![screenshot of sepia effect](screenshots/03-sepia.jpg)
- Resize an image
  ![screenshot of resized image](screenshots/04-resize.jpg)
- Add colored text
  ![screenshot of colored text](screenshots/05-text.jpg)
- Crop
  ![screenshot of cropped image](screenshots/06-crop.jpg)
- Rotate
  ![screenshot of rotated image](screenshots/08-rotate.jpg)
- Mirror
  ![screenshot of vertically mirrored image](screenshots/09-mirror.jpg)

Download: https://gitlab.com/bertyhell/grandshop/-/releases

## Developer docs

### Run the program from the repository:

Install node v22.12 or newer.

On Windows, `@tensorflow/tfjs-node` is compiled from source during install, so you need the Visual Studio C++ build tools and Python 3.
If you have an old `msvs_version` or `python` setting in your global `.npmrc`, override it:

```bash
npm install --msvs_version=2022
```

Run the app in development mode with hot reloading:

```bash
npm run dev
```

Other useful commands:

```bash
npm run typecheck
npm run lint
npm run format
```

## Build a new release

```bash
npm run build:win
```

The installer is written to the `dist` folder.
