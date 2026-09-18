import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SketchLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <SketchLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SketchLogo: Component<IconComponentProps, {}, "">;
type SketchLogo = ReturnType<typeof SketchLogo>;
export default SketchLogo;
