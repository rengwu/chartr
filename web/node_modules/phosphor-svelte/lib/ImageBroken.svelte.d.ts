import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ImageBrokenIcon` instead.
 *
 * @example
 * ```svelte
 * <ImageBroken color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ImageBroken: Component<IconComponentProps, {}, "">;
type ImageBroken = ReturnType<typeof ImageBroken>;
export default ImageBroken;
