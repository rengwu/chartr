import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `OfficeChairIcon` instead.
 *
 * @example
 * ```svelte
 * <OfficeChair color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const OfficeChair: Component<IconComponentProps, {}, "">;
type OfficeChair = ReturnType<typeof OfficeChair>;
export default OfficeChair;
