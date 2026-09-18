import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GrainsSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <GrainsSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GrainsSlash: Component<IconComponentProps, {}, "">;
type GrainsSlash = ReturnType<typeof GrainsSlash>;
export default GrainsSlash;
