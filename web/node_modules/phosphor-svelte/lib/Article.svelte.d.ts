import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArticleIcon` instead.
 *
 * @example
 * ```svelte
 * <Article color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Article: Component<IconComponentProps, {}, "">;
type Article = ReturnType<typeof Article>;
export default Article;
