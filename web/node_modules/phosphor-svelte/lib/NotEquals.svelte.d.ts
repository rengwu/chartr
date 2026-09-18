import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotEqualsIcon` instead.
 *
 * @example
 * ```svelte
 * <NotEquals color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NotEquals: Component<IconComponentProps, {}, "">;
type NotEquals = ReturnType<typeof NotEquals>;
export default NotEquals;
