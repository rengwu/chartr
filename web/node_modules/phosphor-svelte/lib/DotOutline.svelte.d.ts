import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotOutlineIcon` instead.
 *
 * @example
 * ```svelte
 * <DotOutline color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotOutline: Component<IconComponentProps, {}, "">;
type DotOutline = ReturnType<typeof DotOutline>;
export default DotOutline;
