import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BulldozerIcon` instead.
 *
 * @example
 * ```svelte
 * <Bulldozer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bulldozer: Component<IconComponentProps, {}, "">;
type Bulldozer = ReturnType<typeof Bulldozer>;
export default Bulldozer;
