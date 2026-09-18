import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `KeyIcon` instead.
 *
 * @example
 * ```svelte
 * <Key color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Key: Component<IconComponentProps, {}, "">;
type Key = ReturnType<typeof Key>;
export default Key;
