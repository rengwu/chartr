import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandArrowDownIcon` instead.
 *
 * @example
 * ```svelte
 * <HandArrowDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandArrowDown: Component<IconComponentProps, {}, "">;
type HandArrowDown = ReturnType<typeof HandArrowDown>;
export default HandArrowDown;
