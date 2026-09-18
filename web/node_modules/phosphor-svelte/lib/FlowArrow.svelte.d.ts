import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlowArrowIcon` instead.
 *
 * @example
 * ```svelte
 * <FlowArrow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlowArrow: Component<IconComponentProps, {}, "">;
type FlowArrow = ReturnType<typeof FlowArrow>;
export default FlowArrow;
