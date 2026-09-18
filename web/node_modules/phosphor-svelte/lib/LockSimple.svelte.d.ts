import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LockSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <LockSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LockSimple: Component<IconComponentProps, {}, "">;
type LockSimple = ReturnType<typeof LockSimple>;
export default LockSimple;
