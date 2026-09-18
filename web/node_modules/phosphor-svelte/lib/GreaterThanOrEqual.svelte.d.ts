import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GreaterThanOrEqualIcon` instead.
 *
 * @example
 * ```svelte
 * <GreaterThanOrEqual color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GreaterThanOrEqual: Component<IconComponentProps, {}, "">;
type GreaterThanOrEqual = ReturnType<typeof GreaterThanOrEqual>;
export default GreaterThanOrEqual;
