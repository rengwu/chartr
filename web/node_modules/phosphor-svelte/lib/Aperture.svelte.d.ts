import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ApertureIcon` instead.
 *
 * @example
 * ```svelte
 * <Aperture color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Aperture: Component<IconComponentProps, {}, "">;
type Aperture = ReturnType<typeof Aperture>;
export default Aperture;
