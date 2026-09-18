import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudSnowIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudSnow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudSnow: Component<IconComponentProps, {}, "">;
type CloudSnow = ReturnType<typeof CloudSnow>;
export default CloudSnow;
