import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LessThanIcon` instead.
 *
 * @example
 * ```svelte
 * <LessThan color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LessThan: Component<IconComponentProps, {}, "">;
type LessThan = ReturnType<typeof LessThan>;
export default LessThan;
