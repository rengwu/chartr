import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShieldWarningIcon` instead.
 *
 * @example
 * ```svelte
 * <ShieldWarning color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShieldWarning: Component<IconComponentProps, {}, "">;
type ShieldWarning = ReturnType<typeof ShieldWarning>;
export default ShieldWarning;
