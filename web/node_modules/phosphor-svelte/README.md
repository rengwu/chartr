<img src="./meta/phosphor-mark-tight-yellow.png" width="128" align="right" />

# phosphor-svelte

Phosphor is a flexible icon family for interfaces, diagrams, presentations — whatever, really. More icons at [phosphoricons.com](https://phosphoricons.com).

[![npm](https://img.shields.io/npm/v/phosphor-svelte)](https://npm.im/phosphor-svelte)

## Installation

```bash
yarn add --dev phosphor-svelte
```

or

```bash
npm install --save-dev phosphor-svelte
```

## Usage

```html
<script>
  import { HorseIcon, HeartIcon } from "phosphor-svelte";
  // or
  import CubeIcon from "phosphor-svelte/lib/CubeIcon"; // Recommended for faster compiling
</script>

<HorseIcon />
<HeartIcon color="#AE2983" weight="fill" size="{32}" />
<CubeIcon color="teal" weight="duotone" />
```

> [!NOTE]
> Components are exported with an `Icon` suffix (e.g., `CubeIcon`). The original names without the suffix (e.g., `Cube`) are still available but deprecated and will show warnings in your IDE or Code Editor .

> [!WARNING]
> You might encounter slower compilation when importing components using named import (`import { X } from "phosphor-svelte"`).

### Props

- **color?**: `string` – Icon stroke/fill color. Can be any CSS color string, including `hex`, `rgb`, `rgba`, `hsl`, `hsla`, named colors, or the special `currentColor` variable.
- **size?**: `number | string` – Icon height & width. As with standard React elements, this can be a number, or a string with units in `px`, `%`, `em`, `rem`, `pt`, `cm`, `mm`, `in`.
- **weight?**: `"thin" | "light" | "regular" | "bold" | "fill" | "duotone"` – Icon weight/style. Can also be used, for example, to "toggle" an icon's state: a rating component could use Stars with `weight="regular"` to denote an empty star, and `weight="fill"` to denote a filled star.
- **mirrored?**: `boolean` – Flip the icon horizontally. Can be useful in RTL languages where normal icon orientation is not appropriate.

### Context

Apply default style to all icons. Create an IconContext at the root of the app (or anywhere above the icons in the tree) and pass in a configuration object with props to be applied by default to all icons inside context:

```html
<script>
  import IconContext from "phosphor-svelte/lib/IconContext";
  // or
  // import { IconContext } from "phosphor-svelte";

  import CubeIcon from "phosphor-svelte/lib/CubeIcon";
  import HorseIcon from "phosphor-svelte/lib/HorseIcon";
  import HeartIcon from "phosphor-svelte/lib/HeartIcon";
</script>

<IconContext
  values={{ color: 'limegreen', size: 32, mirrored: false, weight: 'bold' }}>
  <HorseIcon /> <!-- I'm lime-green, 32px, and bold! -->
  <HeartIcon /> <!-- Me too! -->
  <CubeIcon color="red" /> <!-- red -->
</IconContext>
```

### Composability

<img src="./meta/cube-rotate.svg" width="128" align="right" />

Components can accept arbitrary SVG elements as children, so long as they are valid children of the `<svg>` element. This can be used to modify an icon with background layers or shapes, filters, animations and more. The children will be placed _below_ the normal icon contents.

The following will cause the Cube icon to rotate and pulse:

```html
<CubeIcon color="darkorchid" weight="duotone">
  <animate
    attributeName="opacity"
    values="0;1;0"
    dur="4s"
    repeatCount="indefinite"
  ></animate>
  <animateTransform
    attributeName="transform"
    attributeType="XML"
    type="rotate"
    dur="5s"
    from="0 0 0"
    to="360 0 0"
    repeatCount="indefinite"
  ></animateTransform>
</CubeIcon>
```

> **Note:** The coordinate space of slotted elements is relative to the contents of the icon `viewBox`, which is a 256x256 square. Only [valid SVG elements](https://developer.mozilla.org/en-US/docs/Web/SVG/Element#SVG_elements_by_category) will be rendered.

### Import Optimizer

A Vite plugin that transforms named import to default import. This will speed up compile times during development.

```diff
<script>
-  import { CubeIcon, HeartIcon, HorseIcon } from "phosphor-svelte";
+  import CubeIcon from "phosphor-svelte/lib/CubeIcon";
+  import HeartIcon from "phosphor-svelte/lib/HeartIcon";
+  import HorseIcon from "phosphor-svelte/lib/HorseIcon";
</script>
```

#### Usage

Add the plugin into your vite config file.

```javascript
// vite.config.ts

import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import { sveltePhosphorOptimize } from "phosphor-svelte/vite";

export default defineConfig({
  plugins: [sveltekit(), sveltePhosphorOptimize()],
});
```

## License

MIT © [Phosphor Icons](https://github.com/phosphor-icons)

## Knowledge

- [Phosphor React](https://github.com/phosphor-icons/phosphor-react/)
- [Carbon Icons Svelte](https://github.com/IBM/carbon-icons-svelte/)
