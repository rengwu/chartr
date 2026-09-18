import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShieldPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <ShieldPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShieldPlus: Component<IconComponentProps, {}, "">;
type ShieldPlus = ReturnType<typeof ShieldPlus>;
export default ShieldPlus;
