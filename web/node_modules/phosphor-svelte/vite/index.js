import MagicString from "magic-string";
import { walk } from "estree-walker";

const EXCLUDE_RE = /\/node_modules\/|\/\.svelte-kit\/|virtual:__sveltekit/;
const CSS_RE = /\.(css|less|sass|scss|styl|stylus|pcss|postcss|sss)(?:$|\?)/;
const HTML_RE = /\.(html|htm)(?:$|\?)/;

function parseId(id) {
  const parts = id.split("?", 2);
  const filename = parts[0];
  const rawQuery = parts[1];

  const query = Object.fromEntries(new URLSearchParams(rawQuery));
  for (const key in query) {
    if (query[key] === "") {
      query[key] = true;
    }
  }

  return { filename, query };
}

/**
 *
 * @returns {import("vite").Plugin}
 */
export function sveltePhosphorOptimize() {
  return {
    name: "vite-plugin-svelte-phosphor-optimize",
    transform(code, id) {
      const { query, filename } = parseId(id);
      if (
        EXCLUDE_RE.test(filename) ||
        CSS_RE.test(filename) ||
        HTML_RE.test(filename) ||
        query.type === "style"
      )
        return;

      const s = new MagicString(code);
      let root = this.parse(code);

      walk(root, {
        enter(node) {
          if (
            node.type === "ImportDeclaration" &&
            node.source.value === "phosphor-svelte"
          ) {
            let content = "";

            for (const specifier of node.specifiers) {
              if (specifier.type === "ImportSpecifier") {
                const fragment = `import ${specifier.local.name} from "phosphor-svelte/lib/${specifier.imported.name}";\n`;
                if (fragment) content += fragment;
              }
            }

            if (content) s.overwrite(node.start, node.end, content);
          }
        },
      });

      return {
        code: s.toString(),
        map: s.generateMap({ hires: true }),
      };
    },
  };
}
