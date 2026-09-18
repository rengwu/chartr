import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FanIcon` instead.
 *
 * @example
 * ```svelte
 * <Fan color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Fan: Component<IconComponentProps, {}, "">;
type Fan = ReturnType<typeof Fan>;
export default Fan;
