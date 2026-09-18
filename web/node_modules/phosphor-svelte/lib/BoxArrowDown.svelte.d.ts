import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BoxArrowDownIcon` instead.
 *
 * @example
 * ```svelte
 * <BoxArrowDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BoxArrowDown: Component<IconComponentProps, {}, "">;
type BoxArrowDown = ReturnType<typeof BoxArrowDown>;
export default BoxArrowDown;
