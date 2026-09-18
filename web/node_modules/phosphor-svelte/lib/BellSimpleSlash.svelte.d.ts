import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BellSimpleSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <BellSimpleSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BellSimpleSlash: Component<IconComponentProps, {}, "">;
type BellSimpleSlash = ReturnType<typeof BellSimpleSlash>;
export default BellSimpleSlash;
