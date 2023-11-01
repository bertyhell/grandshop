# Grandshop

A image editing program that is really easy to use.

Current features:

-   Upscale images (4x)
		![screenshot of upscaled image](screenshots/07-upscale.jpg)
-   Add colored borders
		![screenshot of colored border](screenshots/01-border.jpg)
-   Change the brightness
		![screenshot of brightness](screenshots/02-brightness.jpg)
-   Add a sepia effect
		![screenshot of sepia effect](screenshots/03-sepia.jpg)
-   Resize an image
		![screenshot of resized image](screenshots/04-resize.jpg)
-   Add colored text
		![screenshot of colored text](screenshots/05-text.jpg)
-   Crop
		![screenshot of cropped image](screenshots/06-crop.jpg)
-   Rotate
		![screenshot of rotated image](screenshots/08-rotate.jpg)
-   Mirror
		![screenshot of vertically mirrored image](screenshots/09-mirror.jpg)

Download: https://gitlab.com/bertyhell/grandshop/-/releases

## Developer docs

### Run the program from the repository:

install node v12.14.1

install dependencies

```bash
npm i
electron-rebuild
```

Run the frontend and electron parts of the application:

```bash
npm run electron-start
```

```bash
npm run react-start
```

## Build a new release

```bash
npm run dist
```

Compile the installer using inno setup
