import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PaletteIcon` instead.
 *
 * @example
 * ```svelte
 * <Palette color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Palette: Component<IconComponentProps, {}, "">;
type Palette = ReturnType<typeof Palette>;
export default Palette;
