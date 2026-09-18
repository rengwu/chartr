import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PencilSimpleSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <PencilSimpleSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PencilSimpleSlash: Component<IconComponentProps, {}, "">;
type PencilSimpleSlash = ReturnType<typeof PencilSimpleSlash>;
export default PencilSimpleSlash;
