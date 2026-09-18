import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `OrangeSliceIcon` instead.
 *
 * @example
 * ```svelte
 * <OrangeSlice color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const OrangeSlice: Component<IconComponentProps, {}, "">;
type OrangeSlice = ReturnType<typeof OrangeSlice>;
export default OrangeSlice;
