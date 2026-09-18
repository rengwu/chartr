import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsNineIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsNine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsNine: Component<IconComponentProps, {}, "">;
type DotsNine = ReturnType<typeof DotsNine>;
export default DotsNine;
