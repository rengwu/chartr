import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SunDimIcon` instead.
 *
 * @example
 * ```svelte
 * <SunDim color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SunDim: Component<IconComponentProps, {}, "">;
type SunDim = ReturnType<typeof SunDim>;
export default SunDim;
