import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotSupersetOfIcon` instead.
 *
 * @example
 * ```svelte
 * <NotSupersetOf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NotSupersetOf: Component<IconComponentProps, {}, "">;
type NotSupersetOf = ReturnType<typeof NotSupersetOf>;
export default NotSupersetOf;
