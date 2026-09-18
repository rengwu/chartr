import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LockKeyIcon` instead.
 *
 * @example
 * ```svelte
 * <LockKey color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LockKey: Component<IconComponentProps, {}, "">;
type LockKey = ReturnType<typeof LockKey>;
export default LockKey;
