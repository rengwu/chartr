import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PolygonIcon` instead.
 *
 * @example
 * ```svelte
 * <Polygon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Polygon: Component<IconComponentProps, {}, "">;
type Polygon = ReturnType<typeof Polygon>;
export default Polygon;
