import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShieldCheckIcon` instead.
 *
 * @example
 * ```svelte
 * <ShieldCheck color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShieldCheck: Component<IconComponentProps, {}, "">;
type ShieldCheck = ReturnType<typeof ShieldCheck>;
export default ShieldCheck;
