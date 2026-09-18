import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PanoramaIcon` instead.
 *
 * @example
 * ```svelte
 * <Panorama color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Panorama: Component<IconComponentProps, {}, "">;
type Panorama = ReturnType<typeof Panorama>;
export default Panorama;
