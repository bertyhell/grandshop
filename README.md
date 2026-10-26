# Grandshop

An image editing program that is really easy to use.

Current features:
* Add colored borders
* Add a decorative image border: floral, gold frame, vintage scrollwork, art deco or Christmas holly
* Change the brightness, contrast and colour intensity
* Add a sepia effect, at any strength
* Make an image black and white, fully or partly
* Resize an image
* Add colored text to the image, with an outline or shadow, and drag it into place
* Crop, freely or to a square, 4:3, 3:2 or 16:9
* Rotate by any angle, filling or cutting off the corners
* Mirror horizontally or vertically
* Enlarge with AI
* Improve faces with AI
* Remove the background with AI

Download: https://github.com/bertyhell/grandshop/releases

![screenshot of the ui](screenshot.jpg)

## Developer docs

Grandshop is a native app written in [Rust](https://www.rust-lang.org) with the
[Slint](https://slint.dev) UI toolkit:

| Path | What |
| --- | --- |
| `ui/` | The Slint UI: views, components and the controls for each modification |
| `src/pipeline/` | The image modifications |
| `src/ai/` | Downloading the AI models and running them with ONNX Runtime, in tiles |
| `src/worker.rs` | Background rendering of the preview, with a cache per modification |
| `src/controller.rs` | Connects the UI to the image code: cards, open/save, zoom detail |
| `translations/` | Translations as gettext `.po` files |
| `assets/` | Bundled font, decorative borders and the app icon. The borders are public domain (CC0); each has a `LICENSE.txt` with its source. `scripts/slice-border.py` cuts a new one into pieces |
| `tests/golden/` | Reference images from the earlier Electron/sharp version |

While editing, the modifications run on a copy of at most 2560 px, so sliders react
immediately. Saving runs them on the full resolution image, and zooming in shows the
visible part at full resolution.

### Run the program from the repository

Install Rust with [rustup](https://rustup.rs). On Linux, also install:

```bash
sudo apt install libfontconfig1-dev libxkbcommon-dev
```

Then:

```bash
./run.sh
```

`./run.sh photo.jpg` opens a photo right away. `./build.sh` only builds the program, into
`target/release/grandshop`.

### Tests

```bash
cargo test
```

The golden tests compare every modification with the output of the earlier sharp based
version; `tests/golden/params.json` lists the exact parameters.

### AI models

The AI operations run with [ONNX Runtime](https://onnxruntime.ai) through the `ort` crate,
which links a prebuilt library at build time. They use the graphics card where possible:
DirectML on Windows and Core ML on macOS. Linux runs them on the CPU. The setting "Use the
graphics card for AI operations" switches this off.

The models are not part of the installers. Each one is downloaded the first time it is
needed, checked against its SHA-256 and kept in the app's data folder (on Linux
`~/.local/share/grandshop/models/`). They come from
[facefusion's ONNX exports](https://huggingface.co/facefusion/models-3.0.0) and
[rembg's releases](https://github.com/danielgatis/rembg/releases/tag/v0.0.0):

| Model | Used for | Licence |
| --- | --- | --- |
| 4x-ClearRealityV1 | Enlarge, fast | CC BY-NC-SA 4.0 (non-commercial) |
| Real-ESRGAN x4plus, x2plus | Enlarge, best quality | BSD-3-Clause |
| SCRFD 2.5G (InsightFace) | Improve faces: finding them | Non-commercial research use |
| GFPGAN 1.4 | Improve faces: restoring them | Apache-2.0 |
| BiRefNet (general, lite) | Remove background | MIT |

To try the models: `cargo test --release upscale -- --ignored --nocapture` on the test image,
and `GRANDSHOP_FACES_PHOTO=photo.jpg cargo test --release faces -- --ignored --nocapture` or
`GRANDSHOP_BACKGROUND_PHOTO=photo.jpg cargo test --release background -- --ignored --nocapture`
on a photo of your own.

### Translations

UI texts are marked with `@tr("English text")` in the `.slint` files. The Dutch texts are in
`translations/nl/LC_MESSAGES/grandshop.po`, which POEditor can import and export. To check
that every text is translated:

```bash
python3 -I scripts/check-translations.py
```

To add a language, add `translations/<code>/LC_MESSAGES/grandshop.po` and an entry in
`LANGUAGES` in `src/settings.rs`.

## Build a new release

Releases are built by GitHub Actions ([`.github/workflows/release.yml`](.github/workflows/release.yml)).
Bump `version` in `Cargo.toml`, commit it on `master`, then push a matching tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The workflow checks that the tag is on `master` and matches the `Cargo.toml` version, runs
the tests on each OS and publishes a GitHub release with the installers: a `.deb` and
AppImage for Linux, a `.dmg` for macOS and a setup `.exe` for Windows.

To build the installers for your own OS locally, install the packaging tool once and run
the build script:

```bash
cargo install cargo-packager --locked
./build.sh --package
```

They end up in `target/packages`.
