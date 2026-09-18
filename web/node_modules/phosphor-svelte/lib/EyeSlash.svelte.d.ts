import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EyeSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <EyeSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EyeSlash: Component<IconComponentProps, {}, "">;
type EyeSlash = ReturnType<typeof EyeSlash>;
export default EyeSlash;
