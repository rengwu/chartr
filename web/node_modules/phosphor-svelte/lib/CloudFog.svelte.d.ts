import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudFogIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudFog color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudFog: Component<IconComponentProps, {}, "">;
type CloudFog = ReturnType<typeof CloudFog>;
export default CloudFog;
