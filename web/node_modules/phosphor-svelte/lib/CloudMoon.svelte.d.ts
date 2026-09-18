import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudMoonIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudMoon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudMoon: Component<IconComponentProps, {}, "">;
type CloudMoon = ReturnType<typeof CloudMoon>;
export default CloudMoon;
