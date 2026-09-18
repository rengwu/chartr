import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowCircleUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowCircleUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowCircleUp: Component<IconComponentProps, {}, "">;
type ArrowCircleUp = ReturnType<typeof ArrowCircleUp>;
export default ArrowCircleUp;
