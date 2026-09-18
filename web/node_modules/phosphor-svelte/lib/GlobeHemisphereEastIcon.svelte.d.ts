import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <GlobeHemisphereEastIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GlobeHemisphereEastIcon: Component<IconComponentProps, {}, "">;
type GlobeHemisphereEastIcon = ReturnType<typeof GlobeHemisphereEastIcon>;
export default GlobeHemisphereEastIcon;
