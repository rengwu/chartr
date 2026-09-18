import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BellSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <BellSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BellSlash: Component<IconComponentProps, {}, "">;
type BellSlash = ReturnType<typeof BellSlash>;
export default BellSlash;
