import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ProhibitInsetIcon` instead.
 *
 * @example
 * ```svelte
 * <ProhibitInset color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ProhibitInset: Component<IconComponentProps, {}, "">;
type ProhibitInset = ReturnType<typeof ProhibitInset>;
export default ProhibitInset;
