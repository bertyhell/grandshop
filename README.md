# Grandshop

A image editing program that is really easy to use.

Current features:
* Add colored borders
* Change the brightness
* Add a sepia effect
* Resize an image
* Add text below the image
* Crop

Download: https://gitlab.com/bertyhell/grandshop/-/releases

![screenshot of the ui](screenshot.jpg)

## Developer docs

### Run the program from the repository:

install node 12

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


