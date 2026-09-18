import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GlobeHemisphereEastIcon` instead.
 *
 * @example
 * ```svelte
 * <GlobeHemisphereEast color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GlobeHemisphereEast: Component<IconComponentProps, {}, "">;
type GlobeHemisphereEast = ReturnType<typeof GlobeHemisphereEast>;
export default GlobeHemisphereEast;
