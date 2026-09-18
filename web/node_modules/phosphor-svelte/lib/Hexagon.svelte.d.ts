import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HexagonIcon` instead.
 *
 * @example
 * ```svelte
 * <Hexagon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Hexagon: Component<IconComponentProps, {}, "">;
type Hexagon = ReturnType<typeof Hexagon>;
export default Hexagon;
