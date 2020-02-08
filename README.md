# Grandshop

A image editing program that is really easy to use.

Current features:
* add colored borders
* change the brightness
* add a sepia effect
* resize an image
* add text below the image
* crop

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


