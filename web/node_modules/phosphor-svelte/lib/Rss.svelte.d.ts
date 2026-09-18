import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RssIcon` instead.
 *
 * @example
 * ```svelte
 * <Rss color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Rss: Component<IconComponentProps, {}, "">;
type Rss = ReturnType<typeof Rss>;
export default Rss;
