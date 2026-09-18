import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GlobeSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <GlobeSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GlobeSimple: Component<IconComponentProps, {}, "">;
type GlobeSimple = ReturnType<typeof GlobeSimple>;
export default GlobeSimple;
