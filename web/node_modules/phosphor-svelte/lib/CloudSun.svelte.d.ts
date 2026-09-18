import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudSunIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudSun color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudSun: Component<IconComponentProps, {}, "">;
type CloudSun = ReturnType<typeof CloudSun>;
export default CloudSun;
