import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EraserIcon` instead.
 *
 * @example
 * ```svelte
 * <Eraser color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Eraser: Component<IconComponentProps, {}, "">;
type Eraser = ReturnType<typeof Eraser>;
export default Eraser;
