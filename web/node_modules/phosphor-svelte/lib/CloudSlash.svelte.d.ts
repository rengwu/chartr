import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudSlash: Component<IconComponentProps, {}, "">;
type CloudSlash = ReturnType<typeof CloudSlash>;
export default CloudSlash;
