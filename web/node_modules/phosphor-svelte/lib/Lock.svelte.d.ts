import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LockIcon` instead.
 *
 * @example
 * ```svelte
 * <Lock color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lock: Component<IconComponentProps, {}, "">;
type Lock = ReturnType<typeof Lock>;
export default Lock;
