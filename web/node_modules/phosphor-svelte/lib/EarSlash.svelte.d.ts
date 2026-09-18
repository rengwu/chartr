import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EarSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <EarSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EarSlash: Component<IconComponentProps, {}, "">;
type EarSlash = ReturnType<typeof EarSlash>;
export default EarSlash;
