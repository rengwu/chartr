import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GreaterThanIcon` instead.
 *
 * @example
 * ```svelte
 * <GreaterThan color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GreaterThan: Component<IconComponentProps, {}, "">;
type GreaterThan = ReturnType<typeof GreaterThan>;
export default GreaterThan;
