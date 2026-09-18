import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ResizeIcon` instead.
 *
 * @example
 * ```svelte
 * <Resize color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Resize: Component<IconComponentProps, {}, "">;
type Resize = ReturnType<typeof Resize>;
export default Resize;
