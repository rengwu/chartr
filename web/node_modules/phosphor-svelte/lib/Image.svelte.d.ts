import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ImageIcon` instead.
 *
 * @example
 * ```svelte
 * <Image color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Image: Component<IconComponentProps, {}, "">;
type Image = ReturnType<typeof Image>;
export default Image;
