import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LockOpenIcon` instead.
 *
 * @example
 * ```svelte
 * <LockOpen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LockOpen: Component<IconComponentProps, {}, "">;
type LockOpen = ReturnType<typeof LockOpen>;
export default LockOpen;
