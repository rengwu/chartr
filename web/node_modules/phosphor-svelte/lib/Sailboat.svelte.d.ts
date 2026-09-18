import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SailboatIcon` instead.
 *
 * @example
 * ```svelte
 * <Sailboat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sailboat: Component<IconComponentProps, {}, "">;
type Sailboat = ReturnType<typeof Sailboat>;
export default Sailboat;
