import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudArrowDownIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudArrowDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudArrowDown: Component<IconComponentProps, {}, "">;
type CloudArrowDown = ReturnType<typeof CloudArrowDown>;
export default CloudArrowDown;
