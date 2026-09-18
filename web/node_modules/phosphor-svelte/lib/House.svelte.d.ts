import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HouseIcon` instead.
 *
 * @example
 * ```svelte
 * <House color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const House: Component<IconComponentProps, {}, "">;
type House = ReturnType<typeof House>;
export default House;
