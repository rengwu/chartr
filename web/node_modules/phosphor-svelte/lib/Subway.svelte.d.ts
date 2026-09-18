import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SubwayIcon` instead.
 *
 * @example
 * ```svelte
 * <Subway color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Subway: Component<IconComponentProps, {}, "">;
type Subway = ReturnType<typeof Subway>;
export default Subway;
