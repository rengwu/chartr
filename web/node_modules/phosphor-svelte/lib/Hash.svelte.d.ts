import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HashIcon` instead.
 *
 * @example
 * ```svelte
 * <Hash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Hash: Component<IconComponentProps, {}, "">;
type Hash = ReturnType<typeof Hash>;
export default Hash;
