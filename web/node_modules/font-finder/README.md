# font-finder

[![Travis CI build status](https://travis-ci.org/princjef/font-finder.svg?branch=master)](https://travis-ci.org/princjef/font-finder)
[![codecov](https://codecov.io/gh/princjef/font-finder/branch/master/graph/badge.svg)](https://codecov.io/gh/princjef/font-finder)
[![npm version](https://img.shields.io/npm/v/font-finder.svg)](https://npmjs.org/package/font-finder)

Quickly find system font names and metadata without native dependencies.

```
npm install font-finder
```

## Usage

```js
const fontFinder = require('font-finder');

(async () => {
  console.log(await fontFinder.list());
  // {
  //   Calibri: [
  //     {
  //       path: 'C:/WINDOWS/Fonts/calibri.ttf',
  //       type: 'sansSerif',
  //       weight: 400,
  //       style: 'regular'
  //     },
  //     {
  //       path: 'C:/WINDOWS/Fonts/calibrii.ttf',
  //       type: 'sansSerif',
  //       weight: 400,
  //       style: 'italic'
  //     },
  //     ...
  //   ],
  //   'Courier New': [
  //     {
  //       path: 'C:/WINDOWS/Fonts/courbi.ttf',
  //       type: 'monospace',
  //       weight: 700,
  //       style: 'boldItalic'
  //     },
  //     ...
  //   ],
  //   ...
  // }

  console.log(await fontFinder.listVariants('Calibri'));
  // [
  //   {
  //     path: 'C:/WINDOWS/Fonts/calibri.ttf',
  //     type: 'sansSerif',
  //     weight: 400,
  //     style: 'regular'
  //   },
  //   {
  //     path: 'C:/WINDOWS/Fonts/calibrii.ttf',
  //     type: 'sansSerif',
  //     weight: 400,
  //     style: 'italic'
  //   },
  //   ...
  // ]

  console.log(await fontFinder.get('C:/WINDOWS/Fonts/calibri.ttf'));
  // {
  //   name: 'Calibri',
  //   path: 'C:/WINDOWS/Fonts/calibri.ttf',
  //   type: 'sansSerif',
  //   weight: 400,
  //   style: 'regular'
  // }

})();
```

## API

### `list([options])`

Lists all fonts present on the system, grouped by font name. Currently limited
to ttf and otf fonts. Returns an object where the keys are the font family names
and the values are arrays of metadata for each font variant.

**Params**

 * `options` [*object*] - Options for configuring font retrieval
    * `language` [*string*] - The language to use when fetching font naming
      information. Default: 'en'
    * `concurrency` [*string[]*] - Maximum number of fonts to process
      concurently. You can tweak this to get the maximum performance out of the
      call. Default: 4

### `listVariants(name, [options])`

Lists all variants found for the provided font family. The format of each
variant is the same as for each variant in the `list()` call. If no variants are
found, an empty array is returned.

**Params**

 * `name` [*string*] - The font family name to search for
 * `options` [*object*] - Same as for `list()`

### `get(path, [options])`

Gets metadata for a single font file, returning metadata for the first font
variant found in the file. If there is an error extracting the font, an error is
thrown (unlike in list, where the font is simply ignored). Returns data for a
single font variant. The format is the same as for a single variant in `list()`
and `listVariants()`, but also includes the font's name.

 * `path` [*string*] - Absolute path to the font file to parse
 * `options` [*object*] - Options for configuring font retrieval
    * `language` [*string*] - The language to use when fetching font naming
      information. Default: 'en'

## Contributing

Want to contribute to the project? Go check out the [Contribution 
Guide](CONTRIBUTING.md) for instructions to set up your development 
environment, open an issue and create a pull request.
