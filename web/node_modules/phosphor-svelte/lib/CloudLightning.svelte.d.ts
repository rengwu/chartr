import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudLightningIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudLightning color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudLightning: Component<IconComponentProps, {}, "">;
type CloudLightning = ReturnType<typeof CloudLightning>;
export default CloudLightning;
