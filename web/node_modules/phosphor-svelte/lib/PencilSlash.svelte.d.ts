import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PencilSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <PencilSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PencilSlash: Component<IconComponentProps, {}, "">;
type PencilSlash = ReturnType<typeof PencilSlash>;
export default PencilSlash;
