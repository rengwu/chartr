import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SlideshowIcon` instead.
 *
 * @example
 * ```svelte
 * <Slideshow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Slideshow: Component<IconComponentProps, {}, "">;
type Slideshow = ReturnType<typeof Slideshow>;
export default Slideshow;
