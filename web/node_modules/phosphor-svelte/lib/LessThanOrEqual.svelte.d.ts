import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LessThanOrEqualIcon` instead.
 *
 * @example
 * ```svelte
 * <LessThanOrEqual color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LessThanOrEqual: Component<IconComponentProps, {}, "">;
type LessThanOrEqual = ReturnType<typeof LessThanOrEqual>;
export default LessThanOrEqual;
