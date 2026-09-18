import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UniteIcon` instead.
 *
 * @example
 * ```svelte
 * <Unite color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Unite: Component<IconComponentProps, {}, "">;
type Unite = ReturnType<typeof Unite>;
export default Unite;
