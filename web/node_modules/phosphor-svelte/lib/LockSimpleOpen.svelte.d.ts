import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LockSimpleOpenIcon` instead.
 *
 * @example
 * ```svelte
 * <LockSimpleOpen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LockSimpleOpen: Component<IconComponentProps, {}, "">;
type LockSimpleOpen = ReturnType<typeof LockSimpleOpen>;
export default LockSimpleOpen;
